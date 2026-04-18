import "./style.css";
import NightSky from "./components/NightSky";
import Header from "./components/Header";
import ChatContainer from "./components/ChatContainer";
import IndexManager from "./components/IndexManager";

function App() {
  return (
    <div className="relative h-screen min-h-screen flex flex-col overflow-hidden bg-transparent text-slate-100">
      <NightSky />
      <div className="relative z-10 flex min-h-screen flex-col">
        <Header />
        <IndexManager />
        <main className="flex-1 min-h-0">
          <ChatContainer />
        </main>
      </div>
    </div>
  );
}

export default App;
