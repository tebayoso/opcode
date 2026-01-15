import React from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { getClaudeSyntaxTheme } from "@/lib/claudeSyntaxTheme";
import { useTheme } from "@/hooks";
import { cn } from "@/lib/utils";

interface MarkdownRendererProps {
  content: string;
  className?: string;
  /** Size variant for the prose styling */
  size?: "sm" | "base" | "lg";
  /** Whether to allow full width (no max-width constraint) */
  fullWidth?: boolean;
}

/**
 * Reusable markdown renderer component with syntax highlighting
 * Uses react-markdown with GFM support and Prism for code blocks
 */
export const MarkdownRenderer: React.FC<MarkdownRendererProps> = ({
  content,
  className,
  size = "base",
  fullWidth = true,
}) => {
  const { theme } = useTheme();
  const syntaxTheme = getClaudeSyntaxTheme(theme);

  const sizeClass = {
    sm: "prose-sm",
    base: "prose",
    lg: "prose-lg",
  }[size];

  return (
    <div
      className={cn(
        sizeClass,
        "dark:prose-invert",
        fullWidth && "max-w-none",
        className
      )}
    >
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          code({ node, inline, className: codeClassName, children, ...props }: any) {
            const match = /language-(\w+)/.exec(codeClassName || "");
            return !inline && match ? (
              <SyntaxHighlighter
                style={syntaxTheme}
                language={match[1]}
                PreTag="div"
                {...props}
              >
                {String(children).replace(/\n$/, "")}
              </SyntaxHighlighter>
            ) : (
              <code className={codeClassName} {...props}>
                {children}
              </code>
            );
          },
          // Improve link handling
          a({ node, children, href, ...props }: any) {
            return (
              <a
                href={href}
                target="_blank"
                rel="noopener noreferrer"
                className="text-primary hover:underline"
                {...props}
              >
                {children}
              </a>
            );
          },
          // Better table rendering
          table({ node, children, ...props }: any) {
            return (
              <div className="overflow-x-auto my-4">
                <table className="min-w-full" {...props}>
                  {children}
                </table>
              </div>
            );
          },
          // Better image handling
          img({ node, src, alt, ...props }: any) {
            return (
              <img
                src={src}
                alt={alt || ""}
                className="max-w-full h-auto rounded-md"
                loading="lazy"
                {...props}
              />
            );
          },
          // Better blockquote styling
          blockquote({ node, children, ...props }: any) {
            return (
              <blockquote
                className="border-l-4 border-primary/50 pl-4 italic text-muted-foreground"
                {...props}
              >
                {children}
              </blockquote>
            );
          },
          // Checkbox handling for task lists
          input({ node, type, checked, ...props }: any) {
            if (type === "checkbox") {
              return (
                <input
                  type="checkbox"
                  checked={checked}
                  readOnly
                  className="mr-2 h-4 w-4 rounded border-border"
                  {...props}
                />
              );
            }
            return <input type={type} {...props} />;
          },
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
};

export default MarkdownRenderer;
