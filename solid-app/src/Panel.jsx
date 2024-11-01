import { PORT, IP_ADDR } from './Home'
import { messageEdit } from './Home';
import { vehicles, vehiclesEdit } from './Home';
import { qr, qrEdit } from './Home';
import { For } from 'solid-js';

export function getVehicles() {
  fetch(`http://${IP_ADDR}:${PORT}/api/vehicles`)
    .then(response => response.json())
    .then(vehicles => {
      vehiclesEdit(vehicles);
    })
    .catch(() => {
      messageEdit('Failed to fetch vehicles');
    });
}

export function renderVehicles() {
  let columns = ["id","vehicle_types", "manufacturer", "model", "price", "data", "button"];
  return <table class="min-w-full">
    <tbody>
      <tr>
        <For each={columns}>
          {(col, idx) => <td class="py-3 px-4 font-semibold">{col}</td>}

        </For>
        {/* <td class="py-3 px-4 font-semibold">{`ID`}</td><td class="py-3 px-4 font-semibold">{`Vehicle Type`}</td><td class="py-3 px-4 font-semibold">{`Manufacturer`}</td><td class="py-3 px-4 font-semibold">{`Model`}</td><td class="py-3 px-4 font-semibold">{`Price`}</td><td>{`button`}</td> */}
      </tr>
      <For each={vehicles()}>
        {(vehicle, index) => <tr class="border-b">
          <td class="py-3 px-4 font-semibold">{vehicle.id}</td>
          <td class="py-3 px-4 font-semibold">{vehicle.vehicle_type}</td>
          <td class="py-3 px-4 font-semibold">{vehicle.manufacturer}</td>
          <td class="py-3 px-4 font-semibold">{vehicle.model}</td>
          <td class="py-3 px-4 font-semibold">{vehicle.price}</td>
          <td class="py-3 px-4 font-semibold">{vehicle.data}</td>
          <td><button on:click={[showVehicleQr, vehicle.id]} class="ml-4 bg-green-500 hover:bg-green-700 text-white font-bold py-1 px-2 rounded-lg">{`show qr`}</button></td>
        </tr>}
      </For>
    </tbody>
  </table>

}

export function showVehicleQr(id) {
  fetch(`http://${IP_ADDR}:${PORT}/api/vehicles/qr/${id}`, {
    method: 'GET',
    headers: { 'Content-Type': 'application/octet-stream' }
  })
    .then(response => response.arrayBuffer())
    .then(buffer => {
      const base64String = btoa(
        new Uint8Array(buffer)
          .reduce((data, byte) => data + String.fromCharCode(byte), '')
      );
      qrEdit(`data:image/png;base64,${base64String}`);
      messageEdit('QR code fetched successfully');
    })
    .catch(() => {
      messageEdit('Failed to fetch QR code');
    });
}
