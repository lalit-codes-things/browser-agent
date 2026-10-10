import { Component, ErrorInfo, ReactNode } from "react";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

export class ErrorBoundary extends Component<Props, State> {
  public state: State = {
    hasError: false,
    error: null,
    errorInfo: null,
  };

  public static getDerivedStateFromError(error: Error): Partial<State> {
    return { hasError: true, error };
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error("Uncaught frontend exception:", error, errorInfo);
    this.setState({ errorInfo });
  }

  private handleReload = () => {
    window.location.reload();
  };

  public render() {
    if (this.state.hasError) {
      return (
        <div className="flex h-screen flex-col items-center justify-center bg-bg-0 p-8 font-mono text-primary">
          <div className="w-full max-w-2xl rounded border border-critical bg-surface-0 p-6 shadow-lg">
            <div className="mb-4 flex items-center justify-between border-b border-border pb-3">
              <span className="font-bold tracking-wider text-critical">
                [!] CRITICAL RENDERING FAULT
              </span>
              <span className="micro-annotation text-muted">FAULT_HANDLING_ACTIVE</span>
            </div>

            <p className="mb-4 text-sm text-primary">
              An unhandled exception occurred in the frontend component hierarchy.
            </p>

            <div className="mb-6 rounded bg-bg-1 p-4 text-xs text-critical font-mono overflow-auto max-h-48">
              <div className="font-semibold">{this.state.error?.toString()}</div>
              {this.state.errorInfo?.componentStack && (
                <pre className="mt-2 text-[11px] leading-relaxed opacity-80 whitespace-pre-wrap">
                  {this.state.errorInfo.componentStack}
                </pre>
              )}
            </div>

            <div className="flex justify-end gap-4">
              <button
                onClick={this.handleReload}
                className="rounded border border-critical bg-critical/10 px-4 py-2 text-xs font-semibold text-critical hover:bg-critical/20"
              >
                RELOAD SYSTEM INTERFACE
              </button>
            </div>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
