import React, { useState, useEffect, useRef } from "react";
import { startRecording, stopRecording } from "tauri-plugin-mic-recorder-api";

interface VoiceControlsProps {
  onVoiceInput: (text: string) => void;
  onVoiceOutput?: (text: string) => void;
  isListening: boolean;
  isSpeaking: boolean;
}

const VoiceControls: React.FC<VoiceControlsProps> = ({
  onVoiceInput,
  isListening,
  isSpeaking
}) => {
  const [recording, setRecording] = useState(false);
  const [transcript, setTranscript] = useState("");
  const [error, setError] = useState<string | null>(null);
  const recognitionRef = useRef<any>(null);

  useEffect(() => {
    // Initialize Speech Recognition if available
    const SpeechRecognition = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
    if (SpeechRecognition) {
      recognitionRef.current = new SpeechRecognition();
      recognitionRef.current.continuous = false;
      recognitionRef.current.interimResults = true;
      recognitionRef.current.lang = 'en-US';

      recognitionRef.current.onresult = (event: any) => {
        let interimTranscript = '';
        for (let i = event.resultIndex; i < event.results.length; i++) {
          if (event.results[i].isFinal) {
            interimTranscript += event.results[i][0].transcript;
          } else {
            interimTranscript += event.results[i][0].transcript;
          }
        }
        setTranscript(interimTranscript);
      };

      recognitionRef.current.onerror = (event: any) => {
        console.error('Speech recognition error:', event.error);
        setError(`Speech recognition error: ${event.error}`);
        setRecording(false);
      };

      recognitionRef.current.onend = () => {
        if (recording) {
          recognitionRef.current?.start();
        } else {
          const finalTranscript = transcript.trim();
          if (finalTranscript) {
            onVoiceInput(finalTranscript);
            setTranscript("");
          }
        }
      };
    }

    return () => {
      if (recognitionRef.current) {
        recognitionRef.current.stop();
      }
    };
  }, [onVoiceInput, transcript, recording]);

  const startVoiceInput = async () => {
    try {
      setError(null);
      setRecording(true);
      await startRecording();
      if (recognitionRef.current) {
        recognitionRef.current.start();
      }
    } catch (err) {
      console.error('Failed to start voice input:', err);
      setError(`Failed to start recording: ${String(err)}`);
      setRecording(false);
    }
  };

  const stopVoiceInput = async () => {
    try {
      setRecording(false);
      await stopRecording();
      if (recognitionRef.current) {
        recognitionRef.current.stop();
      }
    } catch (err) {
      console.error('Failed to stop voice input:', err);
      setError(`Failed to stop recording: ${String(err)}`);
    }
  };

  return (
    <div className="flex items-center gap-2 px-3 py-2 bg-[#232a4a] rounded-full">
      {/* Voice Input Button */}
      <button
        onClick={recording ? stopVoiceInput : startVoiceInput}
        disabled={isListening}
        className={`flex items-center justify-center w-10 h-10 rounded-full transition-all duration-200 ${
          recording 
            ? "bg-red-500 hover:bg-red-600" 
            : isListening 
              ? "bg-gray-500 hover:bg-gray-400 opacity-50"
              : "bg-blue-500 hover:bg-blue-600"
        }`}
        title={recording ? "Stop Listening" : "Start Voice Input"}
        aria-label={recording ? "Stop listening" : "Start voice input"}
      >
        {recording ? (
          <svg className="h-5 w-5 text-white animate-pulse" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        ) : (
          <svg className="h-5 w-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z" />
          </svg>
        )}
      </button>

      {/* Transcript Preview */}
      {recording && transcript && (
        <div className="flex-1 min-w-0 text-xs text-slate-300 truncate">
          {transcript}
        </div>
      )}

      {/* Error Display */}
      {error && (
        <div className="text-xs text-red-400">
          {error}
        </div>
      )}

      {/* Voice Output Toggle (visual only) */}
      <button
        disabled
        className={`flex items-center justify-center w-8 h-8 rounded-full transition-all duration-200 ${
          isSpeaking 
            ? "bg-green-500/50 text-white" 
            : "bg-slate-600/50 text-slate-400"
        }`}
        title={isSpeaking ? "Speaking..." : "Voice output disabled"}
        aria-label={isSpeaking ? "Voice output active" : "Voice output disabled"}
      >
        {isSpeaking ? (
          <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15.536 8.464a5 5 0 010 7.072m7.464-11.464a9 9 0 10-12.728 12.728" />
          </svg>
        ) : (
          <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" />
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2" />
          </svg>
        )}
      </button>
    </div>
  );
};

export default VoiceControls;
