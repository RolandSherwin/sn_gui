import { useNavigate } from "react-router-dom";

function Home() {

  let navigate = useNavigate();
  const routeChange = () => {
    let path = `options`;
    navigate(path);
  }

  return (
    <div className="container">

      <div className="">
        <button color="primary" className="px-4" onClick={routeChange}>Options</button>
      </div>

      <h1>Autonomi Client</h1>

    </div>
  );
}
export default Home;
