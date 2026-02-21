import { open } from '@tauri-apps/plugin-dialog';

interface FileLinkerProps {
    onFilesSelected?: (files: string[] | null) => void;
}

function FileLinker({ onFilesSelected }: FileLinkerProps) {
    // This function will ask the user for files
    // Go to https://v2.tauri.app/fr/plugin/dialog/#open-a-file-selector-dialog for more details
    const handleFileSelect = async () => {
        const selectedFiles = await open({
            multiple: true,
            directory: false,
        });
        console.log(selectedFiles);
        onFilesSelected?.(selectedFiles);
    };
    return (
        // Just a button to trigger the file selector
        <button
            type="button"
            onClick={handleFileSelect}
            className="bg-gray-700 hover:bg-gray-600 text-white px-3 py-2 rounded-lg font-semibold disabled:opacity-50"
            title="Select a file"
        >
            📎
        </button>
    );
}


export default FileLinker;