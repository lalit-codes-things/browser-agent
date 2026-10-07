import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { useCallback, memo, useState, useRef } from "react";
import { submitTask } from "../../ipc/client";
import { formatIpcError } from "../../ipc/errors";
import { ScreenshotPreview } from "../../components/preview/ScreenshotPreview";
import { TaskLedger } from "./TaskLedger";
export const ConsoleShell = memo(function ConsoleShell({ state }) {
    const [draft, setDraft] = useState("");
    const inputRef = useRef(null);
    const [localError, setLocalError] = useState(null);
    const handleSubmit = useCallback(async () => {
        if (!draft.trim())
            return;
        setLocalError(null);
        try {
            await submitTask({ task_text: draft.trim() });
            setDraft("");
        }
        catch (err) {
            setLocalError(formatIpcError(err));
        }
    }, [draft]);
    const handleKeyDown = useCallback((e) => {
        if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
            e.preventDefault();
            handleSubmit();
        }
    }, [handleSubmit]);
    return (_jsxs("main", { className: "flex min-h-0 flex-1 flex-col", children: [_jsxs("section", { className: "border-b-1px line border-b px-6 py-4", children: [_jsx("h1", { className: "mb-3 text-primary section-header", children: "CONSOLE" }), _jsx("p", { className: "mb-4 text-muted body-text", children: "Describe the task." }), _jsxs("div", { className: "mb-4 flex max-w-3xl flex-col gap-2", children: [_jsx("textarea", { ref: inputRef, className: "\n              w-full resize-none rounded-none border-1px line bg-surface-1\n              px-3 py-2 text-primary body-text data-mono leading-relaxed\n              placeholder:text-muted\n              focus:outline-none focus:ring-2 focus:ring-white focus:ring-offset-1 focus:ring-offset-bg-0\n              transition-shadow duration-120\n            ", rows: 4, value: draft, onKeyDown: handleKeyDown, onChange: (e) => setDraft(e.target.value), placeholder: "Describe the task.", "aria-label": "Task input" }), localError ? (_jsx("p", { className: "text-sem-violation text-[12px]", children: localError })) : null] }), _jsx("button", { type: "button", onClick: handleSubmit, disabled: !draft.trim(), className: "\n            border-1px line-strong bg-surface-2 px-6 py-2 text-primary\n            font-semibold body-text uppercase tracking-wider-safe\n            disabled:cursor-not-allowed disabled:opacity-50\n            hover:bg-surface-3 active:bg-bg-3\n            transition-colors duration-120\n            max-radius-2\n          ", children: "SUBMIT TASK" })] }), _jsxs("div", { className: "flex-1 flex flex-col lg:flex-row gap-6 p-6", children: [_jsxs("section", { className: "flex-1 min-w-0", children: [_jsx("h2", { className: "mb-3 text-primary section-header", children: "TASK CONSOLE" }), _jsx(TaskLedger, { state: state })] }), _jsxs("section", { className: "w-full max-w-3xl lg:max-w-none", children: [_jsx("h2", { className: "mb-3 text-primary section-header", children: "CONTROLLED BROWSER STATE" }), _jsx(ScreenshotPreview, { state: state })] })] })] }));
});
