import "./style.css";
import NightSky from "./components/NightSky";
import Header from "./components/Header";
import ChatContainer from "./components/ChatContainer";
import IndexManager from "./components/IndexManager";

function App() {
  return (
    <div className="relative min-h-screen flex flex-col items-center justify-center overflow-hidden">
      <NightSky />
      <Header />
      <IndexManager />
      <main className="flex-1 flex flex-col justify-center w-full">
        <ChatContainer />
      </main>
    </div>
  );
}

export default App;
