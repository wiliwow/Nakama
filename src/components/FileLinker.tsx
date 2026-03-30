import { open } from '@tauri-apps/plugin-dialog';
import { readTextFile } from '@tauri-apps/plugin-fs';
import { invoke } from '@tauri-apps/api/core';

interface FileLinkerProps {
  onFilesSelected?: (files: { name: string; content: string }[] | null) => void;
}

const FileLinker = ({ onFilesSelected }: FileLinkerProps) => {

  const handleFileSelect = async () => {
    const selectedFiles = await open({
      multiple: true,
      directory: false,
    });

    if (selectedFiles) {
      const filesArray: string[] = Array.isArray(selectedFiles) ? selectedFiles : [selectedFiles];
      const fileObjects: { name: string; content: string }[] = [];

      try {
        for (const filePath of filesArray) {
          try {
            const content = await readTextFile(filePath);
            const fileName = filePath.split('/').pop() || filePath;
            
            // Index file in RAG system
            await invoke("rag_add_file", { 
              filename: fileName, 
              content 
            });
            
            fileObjects.push({ name: fileName, content });
            console.log(`✓ Indexed file: ${fileName}`);
          } catch (err) {
            console.error(`Error reading file ${filePath}:`, err);
          }
        }
        
        if (fileObjects.length > 0) {
          onFilesSelected?.(fileObjects);
        }
      } catch (err) {
        console.error("Error processing files:", err);
      }
    }
  };

  return (
    <button
      type="button"
      onClick={handleFileSelect}
      className="bg-gray-700 hover:bg-gray-600 text-white px-3 py-2 rounded-lg font-semibold disabled:opacity-50"
      title="Select files to include in your message"
    >
      📎
    </button>
  );
};

export default FileLinker;