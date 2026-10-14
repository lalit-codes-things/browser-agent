interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'ghost';
}

/** App-owned icon whitelist is limited to Undo, Redo, Reload, Bookmarks,
 * Downloads and the three-dot menu control. No other app-owned icons. */
export function Button({
  variant = 'secondary',
  className,
  type = 'button',
  children,
  ...rest
}: ButtonProps) {
  return (
    <button type={type} className={`ba-btn ba-btn--${variant} ${className ?? ''}`} {...rest}>
      {children}
    </button>
  );
}
