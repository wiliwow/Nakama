import React from "react";
import FileListElement from "./FileListElement";

interface FileListerProps {
  files: string[];
}

const FileLister: React.FC<FileListerProps> = ({ files }) => {
  return (
    <div className="flex flex-wrap gap-2">
      {files.map((file, index) => (
        <FileListElement key={index} filePath={file} />
      ))}
    </div>
  );
};

export default FileLister;