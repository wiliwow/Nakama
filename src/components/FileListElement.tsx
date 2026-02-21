import React from "react";

interface FileListerProps {
  filePath: string;
}
//wtf
const FileLister: React.FC<FileListerProps> = ({ filePath }) => {
  const fileName = filePath.split(/[\\/]/).pop() || filePath;

  return (
    <div className="flex items-center gap-2 px-3 py-2 bg-blue-900/50 rounded-lg border border-blue-700 text-blue-100">
      <span className="text-lg">📄</span>
      <span className="truncate flex-1" title={filePath}>
        {fileName}
      </span>
    </div>
  );
};

export default FileLister;
