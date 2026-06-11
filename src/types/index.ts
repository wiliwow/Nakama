// Shared types for the Nakama application

export interface MemoryItem {
  id?: string;
  content: string;
  score?: number;
  namespace?: string;
  created_at?: string;
}

export interface FactItem {
  id?: string;
  fact: string;
  confidence?: number;
  namespace?: string;
}

export interface GoalItem {
  id?: string;
  description: string;
  status: 'in_progress' | 'completed' | 'failed';
  progress?: number;
  namespace?: string;
}

export interface MemorySummary {
  total_episodes: number;
  total_facts: number;
  total_goals: number;
  total_failures: number;
  namespaces: string[];
}

export interface ConversationMessage {
  id?: number;
  role: string;
  content: string;
  timestamp: string;
  metadata?: Record<string, unknown>;
}

export interface Conversation {
  id?: number;
  title: string;
  created_at: string;
  updated_at: string;
  messages: ConversationMessage[];
}

export interface RetrievedPassage {
  id: string;
  score: number;
  content: string;
  source?: string;
}

export interface IndexStatus {
  total_indexed_bytes: number;
  indexed_documents: number;
  indexed_chunks: number;
  last_indexed_at: number | null;
  estimated_index_size_mb: number;
}

export interface HealthStatus {
  embeddings_ok: boolean;
  embeddings_error: string | null;
  llm_ok: boolean;
  llm_error: string | null;
}

export interface ScreenCapture {
  id: number;
  width: number;
  height: number;
  dataUrl: string;
  label?: string;
}

export interface Message {
  id?: number;
  sender: 'user' | 'ai' | 'system';
  text: string;
  timestamp?: string;
  files?: { name: string; content: string }[];
  screenshots?: ScreenCapture[];
}

export interface AutomationMessage {
  sender: string;
  text: string;
}

export interface AppState {
  currentConversation: Conversation | null;
  conversations: Conversation[];
  messages: Message[];
  isLoading: boolean;
  isSpeaking: boolean;
  isListening: boolean;
  screenVisionEnabled: boolean;
  screenCaptures: ScreenCapture[];
  screenCaptureLoading: 'primary' | 'all' | null;
  localMemoryAvailable: boolean;
  memoryInitialized: boolean;
  memoryCount: {
    episodes: number;
    facts: number;
    goals: number;
  };
  indexedFilesCount: number;
  voiceConversationEnabled: boolean;
  files: { name: string; content: string }[];
  indexingFiles: boolean;
  showConversationList: boolean;
  showMemoryPanel: boolean;
  showIndexManager: boolean;
}
