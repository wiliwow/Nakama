import "./style.css";
import { ToastProvider } from "./contexts/ToastContext";
import ChatContainer from "./components/ChatContainer";
import IndexManager from "./components/IndexManager";

function App() {
  return (
    <ToastProvider>
      <div className="relative h-screen min-h-screen flex flex-col overflow-hidden bg-slate-950">
        <IndexManager />
        <main className="flex-1 overflow-hidden">
          <ChatContainer />
        </main>
      </div>
    </ToastProvider>
  );
}

export default App;