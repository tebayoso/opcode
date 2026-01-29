// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod checkpoint;
mod claude_binary;
mod cli_tools;
mod commands;
mod process;

use checkpoint::state::CheckpointState;
use commands::agents::{
    cleanup_finished_processes, create_agent, delete_agent, execute_agent, export_agent,
    export_agent_to_file, fetch_github_agent_content, fetch_github_agents, get_agent,
    get_agent_run, get_agent_run_with_real_time_metrics, get_claude_binary_path,
    get_live_session_output, get_session_output, get_session_status, import_agent,
    import_agent_from_file, import_agent_from_github, init_database, kill_agent_session,
    list_agent_runs, list_agent_runs_with_metrics, list_agents, list_claude_installations,
    list_running_sessions, load_agent_session_history, set_claude_binary_path,
    stream_session_output, update_agent, AgentDb,
};
use commands::claude::{
    cancel_claude_execution, check_auto_checkpoint, check_claude_version, cleanup_old_checkpoints,
    clear_checkpoint_manager, continue_claude_code, create_checkpoint, create_project,
    execute_claude_code, find_claude_md_files, find_global_config_files, fork_from_checkpoint, get_checkpoint_diff,
    get_checkpoint_settings, get_checkpoint_state_stats, get_claude_session_output,
    get_claude_settings, get_home_directory, get_hooks_config, get_project_sessions,
    get_recently_modified_files, get_session_timeline, get_system_prompt, list_checkpoints,
    list_directory_contents, list_projects, list_running_claude_sessions, load_session_history,
    open_new_session, read_claude_md_file, restore_checkpoint, resume_claude_code,
    save_claude_md_file, save_claude_settings, save_system_prompt, search_files,
    track_checkpoint_message, track_session_messages, update_checkpoint_settings,
    update_hooks_config, validate_hook_command, ClaudeProcessState,
};
use commands::mcp::{
    mcp_add, mcp_add_from_claude_desktop, mcp_add_json, mcp_get, mcp_get_server_status, mcp_list,
    mcp_read_project_config, mcp_remove, mcp_reset_project_choices, mcp_save_project_config,
    mcp_serve, mcp_test_connection,
};
use commands::tool_registry::{
    mcp_registry_add_server, mcp_registry_get_server, mcp_registry_list_servers,
    mcp_registry_remove_server, mcp_registry_set_tool_enablement, mcp_registry_test_connection,
    skills_check_updates, skills_install, skills_list_installed, skills_search,
    skills_uninstall, tool_registry_cancel_job, tool_registry_create_job,
    tool_registry_get_installation, tool_registry_get_job, tool_registry_get_tool,
    tool_registry_list_jobs, tool_registry_list_tools, tool_registry_register_tool,
    tool_registry_run_validation, tool_registry_unregister_tool,
    tool_registry_update_installation, tool_registry_update_tool, tool_registry_validate_tool,
};

use commands::file_ops::{
    file_ops_bulk, file_ops_clone_agent, file_ops_clone_skill, file_ops_copy, file_ops_delete,
    file_ops_merge_markdown, file_ops_move, file_ops_preview_merge,
};
use commands::proxy::{apply_proxy_settings, get_proxy_settings, save_proxy_settings};
use commands::storage::{
    storage_delete_row, storage_execute_sql, storage_insert_row, storage_list_tables,
    storage_read_table, storage_reset_database, storage_update_row,
};
use commands::usage::{
    get_session_stats, get_usage_by_date_range, get_usage_details, get_usage_stats,
};
use process::ProcessRegistryState;
use std::sync::Mutex;
use tauri::Manager;

#[cfg(target_os = "macos")]
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};

fn main() {
    // Initialize logger
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Initialize agents database
            let conn = init_database(&app.handle()).expect("Failed to initialize agents database");

            // Load and apply proxy settings from the database
            {
                let db = AgentDb(Mutex::new(conn));
                let proxy_settings = match db.0.lock() {
                    Ok(conn) => {
                        // Directly query proxy settings from the database
                        let mut settings = commands::proxy::ProxySettings::default();

                        let keys = vec![
                            ("proxy_enabled", "enabled"),
                            ("proxy_http", "http_proxy"),
                            ("proxy_https", "https_proxy"),
                            ("proxy_no", "no_proxy"),
                            ("proxy_all", "all_proxy"),
                        ];

                        for (db_key, field) in keys {
                            if let Ok(value) = conn.query_row(
                                "SELECT value FROM app_settings WHERE key = ?1",
                                rusqlite::params![db_key],
                                |row| row.get::<_, String>(0),
                            ) {
                                match field {
                                    "enabled" => settings.enabled = value == "true",
                                    "http_proxy" => {
                                        settings.http_proxy = Some(value).filter(|s| !s.is_empty())
                                    }
                                    "https_proxy" => {
                                        settings.https_proxy = Some(value).filter(|s| !s.is_empty())
                                    }
                                    "no_proxy" => {
                                        settings.no_proxy = Some(value).filter(|s| !s.is_empty())
                                    }
                                    "all_proxy" => {
                                        settings.all_proxy = Some(value).filter(|s| !s.is_empty())
                                    }
                                    _ => {}
                                }
                            }
                        }

                        log::info!("Loaded proxy settings: enabled={}", settings.enabled);
                        settings
                    }
                    Err(e) => {
                        log::warn!("Failed to lock database for proxy settings: {}", e);
                        commands::proxy::ProxySettings::default()
                    }
                };

                // Apply the proxy settings
                apply_proxy_settings(&proxy_settings);
            }

            // Re-open the connection for the app to manage
            let conn = init_database(&app.handle()).expect("Failed to initialize agents database");
            app.manage(AgentDb(Mutex::new(conn)));

            // Initialize checkpoint state
            let checkpoint_state = CheckpointState::new();

            // Set the Claude directory path
            if let Ok(claude_dir) = dirs::home_dir()
                .ok_or_else(|| "Could not find home directory")
                .and_then(|home| {
                    let claude_path = home.join(".claude");
                    claude_path
                        .canonicalize()
                        .map_err(|_| "Could not find ~/.claude directory")
                })
            {
                let state_clone = checkpoint_state.clone();
                tauri::async_runtime::spawn(async move {
                    state_clone.set_claude_dir(claude_dir).await;
                });
            }

            app.manage(checkpoint_state);

            // Initialize process registry
            app.manage(ProcessRegistryState::default());

            // Initialize Claude process state
            app.manage(ClaudeProcessState::default());

            // Get the main window
            let window = app.get_webview_window("main").unwrap();

            // Apply window vibrancy with rounded corners on macOS
            #[cfg(target_os = "macos")]
            {
                // Try different vibrancy materials that support rounded corners
                let materials = [
                    NSVisualEffectMaterial::UnderWindowBackground,
                    NSVisualEffectMaterial::WindowBackground,
                    NSVisualEffectMaterial::Popover,
                    NSVisualEffectMaterial::Menu,
                    NSVisualEffectMaterial::Sidebar,
                ];

                let mut applied = false;
                for material in materials.iter() {
                    if apply_vibrancy(&window, *material, None, Some(12.0)).is_ok() {
                        applied = true;
                        break;
                    }
                }

                if !applied {
                    // Fallback without rounded corners
                    apply_vibrancy(
                        &window,
                        NSVisualEffectMaterial::WindowBackground,
                        None,
                        None,
                    )
                    .expect("Failed to apply any window vibrancy");
                }
            }

            // Open DevTools in debug builds
            #[cfg(debug_assertions)]
            {
                window.open_devtools();
            }

            // Ensure window is visible and focused
            window.show().unwrap_or_default();
            window.set_focus().unwrap_or_default();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Claude & Project Management
            list_projects,
            create_project,
            get_project_sessions,
            get_home_directory,
            get_claude_settings,
            open_new_session,
            get_system_prompt,
            check_claude_version,
            save_system_prompt,
            save_claude_settings,
            find_claude_md_files,
            find_global_config_files,
            read_claude_md_file,
            save_claude_md_file,
            load_session_history,
            execute_claude_code,
            continue_claude_code,
            resume_claude_code,
            cancel_claude_execution,
            list_running_claude_sessions,
            get_claude_session_output,
            list_directory_contents,
            search_files,
            get_recently_modified_files,
            get_hooks_config,
            update_hooks_config,
            validate_hook_command,
            // Checkpoint Management
            create_checkpoint,
            restore_checkpoint,
            list_checkpoints,
            fork_from_checkpoint,
            get_session_timeline,
            update_checkpoint_settings,
            get_checkpoint_diff,
            track_checkpoint_message,
            track_session_messages,
            check_auto_checkpoint,
            cleanup_old_checkpoints,
            get_checkpoint_settings,
            clear_checkpoint_manager,
            get_checkpoint_state_stats,
            // Agent Management
            list_agents,
            create_agent,
            update_agent,
            delete_agent,
            get_agent,
            execute_agent,
            list_agent_runs,
            get_agent_run,
            list_agent_runs_with_metrics,
            get_agent_run_with_real_time_metrics,
            list_running_sessions,
            kill_agent_session,
            get_session_status,
            cleanup_finished_processes,
            get_session_output,
            get_live_session_output,
            stream_session_output,
            load_agent_session_history,
            get_claude_binary_path,
            set_claude_binary_path,
            list_claude_installations,
            export_agent,
            export_agent_to_file,
            import_agent,
            import_agent_from_file,
            fetch_github_agents,
            fetch_github_agent_content,
            import_agent_from_github,
            // Usage & Analytics
            get_usage_stats,
            get_usage_by_date_range,
            get_usage_details,
            get_session_stats,
            // MCP (Model Context Protocol)
            mcp_add,
            mcp_list,
            mcp_get,
            mcp_remove,
            mcp_add_json,
            mcp_add_from_claude_desktop,
            mcp_serve,
            mcp_test_connection,
            mcp_reset_project_choices,
            mcp_get_server_status,
            mcp_read_project_config,
            mcp_save_project_config,
            // Storage Management
            storage_list_tables,
            storage_read_table,
            storage_update_row,
            storage_delete_row,
            storage_insert_row,
            storage_execute_sql,
            storage_reset_database,
            // Slash Commands
            commands::slash_commands::slash_commands_list,
            commands::slash_commands::slash_command_get,
            commands::slash_commands::slash_command_save,
            commands::slash_commands::slash_command_delete,
            // Skills
            commands::skills::skills_list,
            commands::skills::skill_get,
            commands::skills::skill_save,
            commands::skills::skill_delete,
            commands::skills::skill_read_file,
            commands::skills::skill_save_file,
            commands::skills::skill_delete_file,
            // Proxy Settings
            get_proxy_settings,
            save_proxy_settings,
            // File Operations
            file_ops_copy,
            file_ops_move,
            file_ops_preview_merge,
            file_ops_merge_markdown,
            file_ops_bulk,
            file_ops_clone_skill,
            file_ops_clone_agent,
            file_ops_delete,
            // Plugins
            commands::plugins::plugins_list_marketplaces,
            commands::plugins::plugins_add_marketplace,
            commands::plugins::plugins_remove_marketplace,
            commands::plugins::plugins_list_installed,
            commands::plugins::plugins_get_details,
            commands::plugins::plugins_install,
            commands::plugins::plugins_uninstall,
            commands::plugins::plugins_enable,
            commands::plugins::plugins_disable,
            commands::plugins::plugins_fetch_marketplace,
            commands::plugins::plugins_read_readme,
            commands::plugins::plugins_read_component,
            commands::plugins::plugins_save_component,
            commands::plugins::plugins_create,
            commands::plugins::plugins_delete,
            // CLI Tools - Detection
            commands::cli_tools::cli_tools_list,
            commands::cli_tools::cli_tool_get_installations,
            commands::cli_tools::cli_tool_set_preferred,
            commands::cli_tools::cli_tool_get_preferred,
            commands::cli_tools::cli_tools_refresh,
            commands::cli_tools::cli_tool_is_available,
            commands::cli_tools::cli_tool_get_command,
            // CLI Tools - Configuration Management
            commands::cli_tools::cli_tool_list_config_files,
            commands::cli_tools::cli_tool_read_config_file,
            commands::cli_tools::cli_tool_write_config_file,
            commands::cli_tools::cli_tool_get_settings,
            commands::cli_tools::cli_tool_set_setting,
            commands::cli_tools::cli_tool_list_mcp_servers,
            commands::cli_tools::cli_tool_add_mcp_server,
            commands::cli_tools::cli_tool_remove_mcp_server,
            commands::cli_tools::cli_tool_list_agents,
            commands::cli_tools::cli_tool_get_agent,
            commands::cli_tools::cli_tool_execute_cli_command,
            commands::cli_tools::cli_tool_get_config_dir,
            // CLI Tools - Usage Tracking
            commands::cli_tools::cli_tool_track_usage,
            commands::cli_tools::cli_tool_get_usage,
            commands::cli_tools::cli_tool_get_usage_stats,
            commands::cli_tools::cli_tool_clear_usage,
            tool_registry_list_tools,
            tool_registry_get_tool,
            tool_registry_register_tool,
            tool_registry_update_tool,
            tool_registry_unregister_tool,
            tool_registry_validate_tool,
            tool_registry_get_installation,
            tool_registry_update_installation,
            tool_registry_run_validation,
            tool_registry_create_job,
            tool_registry_get_job,
            tool_registry_list_jobs,
            tool_registry_cancel_job,
            skills_search,
            skills_install,
            skills_uninstall,
            skills_list_installed,
            skills_check_updates,
            mcp_registry_list_servers,
            mcp_registry_get_server,
            mcp_registry_add_server,
            mcp_registry_remove_server,
            mcp_registry_set_tool_enablement,
            mcp_registry_test_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
