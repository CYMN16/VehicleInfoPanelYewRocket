import styles from './App.module.css';
import { createSignal, createEffect } from 'solid-js';


export const IP_ADDR = '192.168.1.20'; // Change this to your IP
export const PORT = '8000'; // Change this to your port
export const [message, messageEdit] = createSignal("");
export const [messageVisible, setMessageVisible] = createSignal(false);
export const [vehicles, vehiclesEdit] = createSignal([]);
export const [qr, qrEdit] = createSignal("");

import { getVehicles, renderVehicles } from './Panel'

createEffect(() => {
  if (message()) {
    setMessageVisible(true);
    setTimeout(() => {
      setMessageVisible(false);
      messageEdit("");
    }, 5000); // Hide the message after 5 seconds
  }
});

export default function Home() {
  getVehicles();
  return (<div class={styles.App}>
      <header class={styles.header}>
        <img src={qr()} class={styles.logo} alt="logo" />
        {messageVisible() ? <p class="bold shadow-md text-green-700">{message()}</p> : <></>}
        <div class="bg-white shadow-md rounded-lg p-6 text-gray-700">
         {renderVehicles()}
        </div>

        <button on:click={getVehicles}>fetch vehicles</button>
      </header>
    </div>
  );
};