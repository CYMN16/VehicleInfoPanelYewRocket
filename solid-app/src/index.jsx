/* @refresh reload */
import './index.css';
import { render } from "solid-js/web";
import { Router, Route } from "@solidjs/router";
import Home from './Home';
import VehicleInfo from './VehicleInfo';

const App = (props) => {
  return <>
    <h1>Solid Vehicle Info</h1>
  </>
};

render(
  () => (
    <Router root={App}>
      <Route path="/" component={Home}></Route>
      <Route path="/vehicle" component={VehicleInfo} />
    </Router>
  ),
  document.getElementById("root")
);