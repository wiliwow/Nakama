import React from "react";
import Button from "./ui/Button";
import Badge from "./ui/Badge";

interface FileStagerProps {
  files: { name: string; content: string }[];
  indexingFiles: boolean;
  onIndexFiles: () => void;
  onRemoveFile?: (index: number) => void;
}

const FileStager: React.FC<FileStagerProps> = ({ files, indexingFiles, onIndexFiles, onRemoveFile }) => {
  if (files.length === 0) return null;

  return (
    <div className="border-t border-slate-800/30 bg-slate-900/20 px-4 py-3 backdrop-blur-sm">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-2">
          <Badge count={files.length} variant="warning" />
          <span className="text-xs text-slate-300">Staged Files</span>
        </div>
        <Button
          variant="secondary"
          size="sm"
          onClick={onIndexFiles}
          disabled={indexingFiles}
        >
          {indexingFiles ? "Indexing..." : "Index Files"}
        </Button>
      </div>

      {/* File pills */}
      <div className="flex flex-wrap gap-2 mt-2">
        {files.map((file, index) => (
          <span
            key={index}
            className="group inline-flex items-center gap-1 rounded-full bg-slate-800 px-3 py-1 text-xs text-slate-200 border border-slate-700 hover:border-blue-600/50 transition-all"
          >
            <span>📄</span>
            <span>{file.name}</span>
            {onRemoveFile && (
              <button
                onClick={() => onRemoveFile(index)}
                className="ml-0.5 text-slate-400 hover:text-red-400 transition-colors p-0.5 rounded-full hover:bg-slate-700"
                aria-label={`Remove ${file.name}`}
              >
                <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            )}
          </span>
        ))}
      </div>
    </div>
  );
};

export default FileStager;
