import { Routes, Route } from "react-router-dom";
import "./App.css";
import Home from "./home";
import Options from "./options";

function App() {

  return (
    <div>
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/options" element={<Options />} />
      </Routes>
    </div>
  );
}
export default App;
