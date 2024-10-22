use base64::engine::{general_purpose::STANDARD, Engine as _};
use gloo::net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

const IP_ADDR: &str = "192.168.1.20"; //"127.0.0.1";
const PORT: &str = "8000";

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/:id")]
    Info { id: String },
}

#[function_component(App)]
fn main_app() -> Html {
    let vehicle_state = use_state(|| {
        (
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            None as Option<i32>,
        )
    });
    let message = use_state(|| "".to_string());
    let search_text = use_state(|| "".to_string());
    let vehicles = use_state(Vec::new);
    let suggestion_list = use_state(Vec::new);
    let qr_img = use_state(|| "".to_string());

    let get_vehicles = {
        let vehicles = vehicles.clone();
        let message = message.clone();

        Callback::from(move |_| {
            let vehicles = vehicles.clone();
            let message = message.clone();
            spawn_local(async move {
                match Request::get(&format!("http://{}:{}/api/vehicles", IP_ADDR, PORT))
                    .send()
                    .await
                {
                    Ok(resp) if resp.ok() => {
                        let fetched_vehicles: Vec<Vehicle> = resp.json().await.unwrap_or_default();
                        vehicles.set(fetched_vehicles);
                    }
                    _ => message.set("Failed to fetch vehicles".into()),
                }
            });
        })
    };

    let fuzzy_search_vehicles = {
        let vehicles = vehicles.clone();
        let search_text = search_text.clone();
        let message = message.clone();

        Callback::from(move |_| {
            let vehicles = vehicles.clone();
            let search_text = (*search_text).clone();
            let message = message.clone();
            spawn_local(async move {
                match Request::get(&format!(
                    "http://{}:{}/api/vehicles/search/{}",
                    IP_ADDR, PORT, search_text
                ))
                .send()
                .await
                {
                    Ok(resp) if resp.ok() => {
                        let fetched_vehicles: Vec<Vehicle> = resp.json().await.unwrap_or_default();
                        vehicles.set(fetched_vehicles);
                    }
                    _ => message.set("Failed to fetch vehicles".into()),
                }
            });
        })
    };

    let search_unique_cols_vehicles = {
        let suggestion_list = suggestion_list.clone();
        let message = message.clone();

        Callback::from(move |col_name: String| {
            let suggestion_list = suggestion_list.clone();
            let message = message.clone();
            spawn_local(async move {
                match Request::get(&format!(
                    "http://{}:{}/api/vehicles/search/unique/{}",
                    IP_ADDR, PORT, col_name
                ))
                .send()
                .await
                {
                    Ok(resp) if resp.ok() => {
                        let fetched_rows: Vec<String> = resp.json().await.unwrap_or_default();
                        suggestion_list.set(fetched_rows);
                    }
                    _ => message.set("Failed to fetch unique rows from column".into()),
                }
            });
        })
    };

    let create_vehicle = {
        let vehicle_state = vehicle_state.clone();
        let message = message.clone();
        let search_text = search_text.clone();
        let get_vehicles = get_vehicles.clone();
        let fuzzy_search_vehicles = fuzzy_search_vehicles.clone();

        Callback::from(move |_| {
            let (vehicle_type, manufacturer, model, price, data, _) = (*vehicle_state).clone();
            let vehicle_state = vehicle_state.clone();
            let message = message.clone();
            let search_text = search_text.clone();
            let get_vehicles = get_vehicles.clone();
            let fuzzy_search_vehicles = fuzzy_search_vehicles.clone();

            spawn_local(async move {
                let vehicle_data = serde_json::json!({"vehicle_type": vehicle_type, "manufacturer": manufacturer, "model": model, "price": price, "data": data});
                let response = Request::post(&format!("http://{}:{}/api/vehicles", IP_ADDR, PORT))
                    .header("Content-Type", "application/json")
                    .body(vehicle_data.to_string())
                    .send()
                    .await;
                match response {
                    Ok(resp) if resp.ok() => {
                        message.set("Vehicle created successfully".into());
                        if (*search_text).is_empty() {
                            get_vehicles.emit(())
                        } else {
                            fuzzy_search_vehicles.emit(())
                        };
                    }
                    _ => message.set("Failed to create vehicle".into()),
                }
                vehicle_state.set((
                    "".to_string(),
                    "".to_string(),
                    "".to_string(),
                    "".to_string(),
                    "".to_string(),
                    None,
                ));
            });
        })
    };

    let update_vehicle = {
        let vehicle_state = vehicle_state.clone();
        let search_text = search_text.clone();
        let message = message.clone();
        let get_vehicles = get_vehicles.clone();
        let fuzzy_search_vehicles = fuzzy_search_vehicles.clone();

        Callback::from(move |_| {
            let (vehicle_type, manufacturer, model, price, data, edited_vehicle_id) =
                (*vehicle_state).clone();
            let vehicle_state = vehicle_state.clone();
            let search_text = search_text.clone();
            let message = message.clone();
            let get_vehicles = get_vehicles.clone();
            let fuzzy_search_vehicles = fuzzy_search_vehicles.clone();

            if let Some(id) = edited_vehicle_id {
                spawn_local(async move {
                    let response =
                        Request::put(&format!("http://{}:{}/api/vehicles/{}", IP_ADDR, PORT, id))
                            .header("Content-Type", "application/json")
                            .body(
                                serde_json::to_string(&(
                                    id,
                                    vehicle_type.as_str(),
                                    manufacturer.as_str(),
                                    model.as_str(),
                                    price.as_str(),
                                    data.as_str(),
                                ))
                                .unwrap(),
                            )
                            .send()
                            .await;
                    match response {
                        Ok(resp) if resp.ok() => {
                            message.set("Vehicle updated successfully".into());
                            if (*search_text).is_empty() {
                                get_vehicles.emit(())
                            } else {
                                fuzzy_search_vehicles.emit(())
                            };
                        }
                        _ => message.set("Failed to update vehicle".into()),
                    }
                    vehicle_state.set((
                        "".to_string(),
                        "".to_string(),
                        "".to_string(),
                        "".to_string(),
                        "".to_string(),
                        None,
                    ));
                });
            }
        })
    };

    let delete_vehicle = {
        let message = message.clone();
        let get_vehicles = get_vehicles.clone();

        Callback::from(move |id: i32| {
            let message = message.clone();
            let get_vehicles = get_vehicles.clone();

            spawn_local(async move {
                let response =
                    Request::delete(&format!("http://{}:{}/api/vehicles/{}", IP_ADDR, PORT, id))
                        .send()
                        .await;

                match response {
                    Ok(resp) if resp.ok() => {
                        message.set("Vehicle deleted successfully".into());
                        get_vehicles.emit(());
                    }
                    _ => message.set("Failed to delete vehicle".into()),
                }
            });
        })
    };

    let edit_vehicle = {
        let vehicle_state = vehicle_state.clone();
        let vehicles = vehicles.clone();
        Callback::from(move |id: i32| {
            if let Some(vehicle) = vehicles.iter().find(|v| v.id == id) {
                vehicle_state.set((
                    vehicle.vehicle_type.clone(),
                    vehicle.manufacturer.clone(),
                    vehicle.model.clone(),
                    vehicle.price.clone(),
                    vehicle.data.clone(),
                    Some(id),
                ));
            };
        })
    };

    let show_vehicle_qr = {
        let message = message.clone();
        let qr_img = qr_img.clone();

        Callback::from(move |id: i32| {
            let message = message.clone();
            let qr_img = qr_img.clone();

            spawn_local(async move {
                let response = Request::get(&format!(
                    "http://{}:{}/api/vehicles/qr/{}",
                    IP_ADDR, PORT, id
                ))
                .send()
                .await;

                match response {
                    Ok(resp) if resp.ok() => {
                        message.set("Successfully got QR code".into());
                        qr_img.set(STANDARD.encode(resp.binary().await.unwrap()));
                    }
                    _ => message.set("Failed to fetch qr code".into()),
                }
            });
        })
    };

    // search_unique_cols_vehicles.clone().emit("vehicle_type".to_string());
    let id_vec = vec!["vehicle_types", "manufacturer", "model", "price", "data"];

    html! (
        <div class="container mx-auto p-4">

    // <!-- Navigation Bar -->
    <nav class="bg-white dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700">
        <div class="max-w-screen-xl mx-auto flex flex-wrap items-center justify-between p-4">
            <span class="text-2xl font-semibold dark:text-white">{"CYMN"}</span>
            <div class="flex md:order-2 space-x-2">
            // <!-- Search and Menu for Mobile -->
                // <!-- Mobile Search Icon (Toggles Search Field) -->
                <button id="mobile-search-btn" class="md:hidden p-2.5 text-gray-500 dark:text-gray-400 rounded-lg focus:outline-none focus:ring-4 dark:focus:ring-gray-700">
                    <svg class="w-5 h-5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 20 20">
                        <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                            d="m19 19-4-4m0-7A7 7 0 1 1 1 8a7 7 0 0 1 14 0Z" />
                    </svg>
                </button>
    
                // <!-- Search Input for Desktop -->
                <div class="hidden md:block relative">
                    <input type="text" placeholder="Search..." id="desktop-search"
                        class="pl-10 p-2.5 w-full text-sm border-gray-300 rounded-lg bg-gray-50 dark:bg-gray-700 dark:border-gray-600 dark:text-white focus:ring-blue-500 focus:border-blue-500"/>
                    <button onclick={fuzzy_search_vehicles.clone().reform(|_| {})}
                        class="absolute inset-y-0 right-0 px-4 py-2 bg-gray-500 hover:bg-gray-700 text-white font-bold rounded-lg">
                        {"Search"}
                    </button>
                </div>
            </div>
    
            // <!-- Search Input for Mobile (Initially Hidden) -->
            <div id="mobile-search" class="hidden w-full mt-2 md:hidden">
                <input type="text" placeholder="Search..." id="mobile-search-input"
                    class="w-full pl-10 p-2.5 text-sm border-gray-300 rounded-lg bg-gray-50 dark:bg-gray-700 dark:border-gray-600 dark:text-white focus:ring-blue-500 focus:border-blue-500"/>
                <button onclick={fuzzy_search_vehicles.clone().reform(|_| {})}
                    class="mt-2 w-full py-2 bg-gray-500 hover:bg-gray-700 text-white font-bold rounded-lg">
                    {"Search"}
                </button>
            </div>
        </div>
    </nav>

    // <!-- Vehicle Management Section -->
    <div class="w-full p-4">
        <h1 class="text-3xl font-bold text-blue-500 mb-6 text-center">{"Vehicle Management"}</h1>
                {for id_vec.iter().map(|id| {
                    html!(<datalist id={*id}>
                    {for (*suggestion_list).iter().map(|row| {
                        html!(<option value={format!("{}", row)}/>)
                    })}
                </datalist>)
                })}

        // <!-- Form for Vehicle Inputs -->
        <div class="mb-4 grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-4">
            <input list="vehicle_types" placeholder="Vehicle Type" value={vehicle_state.0.clone()} 
                class="border rounded-lg p-2.5"
                onfocus={search_unique_cols_vehicles.clone().reform(|_| {"vehicle_type".to_string()})} 
                oninput={Callback::from({ let vehicle_state = vehicle_state.clone(); move |e: InputEvent| {
                    let input = e.target_dyn_into::<web_sys::HtmlInputElement>().unwrap();
                    vehicle_state.set((input.value(), vehicle_state.1.clone(), vehicle_state.2.clone(), vehicle_state.3.clone(), vehicle_state.4.clone(), vehicle_state.5));
                }})}/>

            <input list="manufacturer" placeholder="Manufacturer" value={vehicle_state.1.clone()} 
                class="border rounded-lg p-2.5"
                onfocus={search_unique_cols_vehicles.clone().reform(|_| {"manufacturer".to_string()})} 
                oninput={Callback::from({ let vehicle_state = vehicle_state.clone(); move |e: InputEvent| {
                    let input = e.target_dyn_into::<web_sys::HtmlInputElement>().unwrap();
                    vehicle_state.set((vehicle_state.0.clone(), input.value(), vehicle_state.2.clone(), vehicle_state.3.clone(), vehicle_state.4.clone(), vehicle_state.5));
                }})}/>

            <input list="model" placeholder="Model" value={vehicle_state.2.clone()} 
                class="border rounded-lg p-2.5"
                onfocus={search_unique_cols_vehicles.clone().reform(|_| {"model".to_string()})} 
                oninput={Callback::from({ let vehicle_state = vehicle_state.clone(); move |e: InputEvent| {
                    let input = e.target_dyn_into::<web_sys::HtmlInputElement>().unwrap();
                    vehicle_state.set((vehicle_state.0.clone(), vehicle_state.1.clone(), input.value(), vehicle_state.3.clone(), vehicle_state.4.clone(), vehicle_state.5));
                }})}/>

            <input list="price" placeholder="Price" value={vehicle_state.3.clone()} 
                class="border rounded-lg p-2.5"
                onfocus={search_unique_cols_vehicles.clone().reform(|_| {"price".to_string()})} 
                oninput={Callback::from({ let vehicle_state = vehicle_state.clone(); move |e: InputEvent| {
                    let input = e.target_dyn_into::<web_sys::HtmlInputElement>().unwrap();
                    vehicle_state.set((vehicle_state.0.clone(), vehicle_state.1.clone(), vehicle_state.2.clone(), input.value(), vehicle_state.4.clone(), vehicle_state.5));
                }})}/>

            <input list="data" placeholder="Data" value={vehicle_state.4.clone()} 
                class="border rounded-lg p-2.5"
                onfocus={search_unique_cols_vehicles.clone().reform(|_| {"data".to_string()})} 
                oninput={Callback::from({ let vehicle_state = vehicle_state.clone(); move |e: InputEvent| {
                    let input = e.target_dyn_into::<web_sys::HtmlInputElement>().unwrap();
                    vehicle_state.set((vehicle_state.0.clone(), vehicle_state.1.clone(), vehicle_state.2.clone(), vehicle_state.3.clone(), input.value(), vehicle_state.5));
                }})}/>

            <button onclick={if vehicle_state.5.is_some() { update_vehicle.clone() } else { create_vehicle.clone() }}
                class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-lg">
                { if vehicle_state.5.is_some() {"Update Vehicle"} else {"Create Vehicle"}}
            </button>
        </div>

        // <!-- Messages -->
        if !message.is_empty() {
            <p class="text-green-500 text-center">{&*message}</p>
        }

        // <!-- Fetch Vehicles Button -->
        <div class="text-center">
            <button onclick={get_vehicles.reform(|_| {})}
                class="bg-gray-500 hover:bg-gray-700 text-white font-bold py-2 px-4 rounded-lg">
                {"Fetch Vehicle List"}
            </button>
        </div>

        // <!-- QR Image -->
        <div class="text-center my-4">
            <img class="inline-block" src={format!("data:image/png;base64,{}", qr_img.clone().to_string())}/>
        </div>

        // <!-- Vehicle List -->
        <h2 class="text-2xl font-bold text-gray-700 mb-4 text-center">{"Vehicle List"}</h2>
        <ul class="list-disc pl-5">
            {for (*vehicles).iter().map(|vehicle| {
                let vehicle_id = vehicle.id;
                html!(
                    <li class="mb-2">
                        <span class="font-semibold">
                            { format!("ID: {}, Vehicle Type: {}, Manufacturer: {}, Model: {}, Price: {}, Data: {}", 
                            vehicle.id, vehicle.vehicle_type, vehicle.manufacturer, vehicle.model, vehicle.price, vehicle.data)}
                        </span>
                        <button onclick={delete_vehicle.clone().reform(move |_| vehicle_id)}
                            class="ml-4 bg-red-500 hover:bg-red-700 text-white font-bold py-1 px-2 rounded-lg">
                            {"Delete"}
                        </button>
                        <button onclick={edit_vehicle.clone().reform(move |_| vehicle_id)}
                            class="ml-4 bg-yellow-500 hover:bg-yellow-700 text-white font-bold py-1 px-2 rounded-lg">
                            {"Edit"}
                        </button>
                        <button onclick={show_vehicle_qr.clone().reform(move |_| vehicle_id)}
                            class="ml-4 bg-green-500 hover:bg-green-700 text-white font-bold py-1 px-2 rounded-lg">
                            {"Generate QR"}
                        </button>
                    </li>
                )
            })}
        </ul>
    </div>

</div>


    //     <div class="container max-w-full m-0 p-0">

    //     <nav class="bg-white border-gray-200 dark:bg-gray-900">
    //         <div class="max-w-screen-xl flex flex-wrap items-center justify-between mx-auto p-4">
    //             <span class="self-center text-2xl font-semibold whitespace-nowrap dark:text-white">{"CYMN"}</span>
    //             <div class="flex md:order-2">
    //                 <button type="button" data-collapse-toggle="navbar-search" aria-controls="navbar-search"
    //                     aria-expanded="false"
    //                     class="md:hidden text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 focus:outline-none focus:ring-4 focus:ring-gray-200 dark:focus:ring-gray-700 rounded-lg text-sm p-2.5 me-1">
    //                     <svg class="w-5 h-5" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none"
    //                         viewBox="0 0 20 20">
    //                         <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
    //                             d="m19 19-4-4m0-7A7 7 0 1 1 1 8a7 7 0 0 1 14 0Z" />
    //                     </svg>
    //                     <span class="sr-only">{"Search"}</span>
    //                 </button>
    //                 <div class="relative hidden md:block">
    //                     <div class="absolute inset-y-0 start-0 flex items-center ps-3 pointer-events-none">
    //                         <svg class="w-4 h-4 text-gray-500 dark:text-gray-400" aria-hidden="true"
    //                             xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 20 20">
    //                             <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
    //                                 d="m19 19-4-4m0-7A7 7 0 1 1 1 8a7 7 0 0 1 14 0Z" />
    //                         </svg>
    //                         <span class="sr-only">{"Search icon"}</span>
    //                     </div>
    //                     <div class="flex md:order-2 justify-between">
    //                         <input type="text" id="search-navbar"
    //                             class="block w-full p-2 ps-10 text-sm text-gray-900 border border-gray-300 rounded-lg bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
    //                             value={(*search_text).clone()} oninput={ Callback::from({ let search_text=search_text.clone();
    //                                 move |e: InputEvent| { let input=e.target_dyn_into::<web_sys::HtmlInputElement>().unwrap();
    //                                     search_text.set(input.value());
    //                                 }
    //                             })} />
    //                             <button onclick={fuzzy_search_vehicles.clone().reform(|_| {})}
    //                                 class="bg-gray-500 hovering:bg-gray-700 text-white font-bold mx-4 py-2 px-4 rounded">
    //                                 { "Search" }
    //                             </button>
    //                     </div>
    //                 </div>
    //                 <button data-collapse-toggle="navbar-search" type="button"
    //                     class="inline-flex items-center p-2 w-10 h-10 justify-center text-sm text-gray-500 rounded-lg md:hidden hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-gray-200 dark:text-gray-400 dark:hover:bg-gray-700 dark:focus:ring-gray-600"
    //                     aria-controls="navbar-search" aria-expanded="false">
    //                     <span class="sr-only">{"Open main menu"}</span>
    //                     <svg class="w-5 h-5" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none"
    //                         viewBox="0 0 17 14">
    //                         <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
    //                             d="M1 1h15M1 7h15M1 13h15" />
    //                     </svg>
    //                 </button>
    //             </div>
    //             <div class="items-center justify-between hidden w-full md:flex md:w-auto md:order-1" id="navbar-search">
    //                 <div class="relative mt-3 md:hidden">
    //                     <div class="absolute inset-y-0 start-0 flex items-center ps-3 pointer-events-none">
    //                         <svg class="w-4 h-4 text-gray-500 dark:text-gray-400" aria-hidden="true"
    //                             xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 20 20">
    //                             <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
    //                                 d="m19 19-4-4m0-7A7 7 0 1 1 1 8a7 7 0 0 1 14 0Z" />
    //                         </svg>
    //                     </div>
    //                     <input type="text" id="search-navbar"
    //                         class="block w-full p-2 ps-10 text-sm text-gray-900 border border-gray-300 rounded-lg bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
    //                         value={(*search_text).clone()} oninput={ Callback::from({ let search_text=search_text.clone();
    //                         move |e: InputEvent| { let input=e.target_dyn_into::<web_sys::HtmlInputElement>().unwrap();
    //                     search_text.set(input.value());
    //                     }
    //                     })} />
    //                     <button onclick={fuzzy_search_vehicles.clone().reform(|_| {})}
    //                     class="bg-gray-500 hovering:bg-gray-700 text-white font-bold py-2 px-4 rounded">
    //                     { "Search" }
    //                 </button>
    //                 </div>
    //             </div>
    //         </div>
    //     </nav>

    //     <div class="w-max p-5">

    //         <h1 class="text-4x1 font-bold text-blue-500 mb-4"> {"Vehicle Management"} </h1>
    //         <div class="mb-4">
    //             {for id_vec.iter().map(|id| {
    //                 html!(<datalist id={*id}>
    //                 {for (*suggestion_list).iter().map(|row| {
    //                     html!(<option value={format!("{}", row)}/>)
    //                 })}
    //             </datalist>)
    //             })}

    //             <input list="vehicle_types" placeholder="Vehicle Type" value={vehicle_state.0.clone()} onfocus={search_unique_cols_vehicles.clone().reform(|_| {"vehicle_type".to_string()})} onkeypress={search_unique_cols_vehicles.clone().reform(|_| {"vehicle_type".to_string()})} oninput={
    //                 Callback::from({ let
    //                 vehicle_state=vehicle_state.clone(); move |e: InputEvent| {
    //                     let input=e.target_dyn_into::<web_sys::HtmlInputElement>().unwrap();
    //                     vehicle_state.set((input.value(), vehicle_state.1.clone(), vehicle_state.2.clone(), vehicle_state.3.clone(),
    //                         vehicle_state.4.clone(), vehicle_state.5));
    //             }
    //             })}
    //             class="border rounded px-4 py-2 mr-2"
    //             />

    //             <input list="manufacturer" placeholder="Manufacturer" value={vehicle_state.1.clone()} onfocus={search_unique_cols_vehicles.clone().reform(|_| {"manufacturer".to_string()})} onkeypress={search_unique_cols_vehicles.clone().reform(|_| {"manufacturer".to_string()})} oninput={ Callback::from({ let
    //                 vehicle_state=vehicle_state.clone(); move |e: InputEvent| { let
    //                 input=e.target_dyn_into::<web_sys::HtmlInputElement>().unwrap();
    //             vehicle_state.set((vehicle_state.0.clone(), input.value(), vehicle_state.2.clone(), vehicle_state.3.clone(),
    //             vehicle_state.4.clone(), vehicle_state.5));
    //             }
    //             })}
    //             class="border rounded px-4 py-2 mr-2"
    //             />
    //             <input list="model" placeholder="Model" value={vehicle_state.2.clone()} onfocus={search_unique_cols_vehicles.clone().reform(|_| {"model".to_string()})} onkeypress={search_unique_cols_vehicles.clone().reform(|_| {"model".to_string()})} oninput={ Callback::from({ let
    //                 vehicle_state=vehicle_state.clone(); move |e: InputEvent| { let
    //                 input=e.target_dyn_into::<web_sys::HtmlInputElement>().unwrap();
    //             vehicle_state.set((vehicle_state.0.clone(), vehicle_state.1.clone(), input.value(), vehicle_state.3.clone(),
    //             vehicle_state.4.clone(), vehicle_state.5));
    //             }
    //             })}
    //             class="border rounded px-4 py-2 mr-2"
    //             />
    //             <input list="price" placeholder="Price" value={vehicle_state.3.clone()} onfocus={search_unique_cols_vehicles.clone().reform(|_| {"price".to_string()})} onkeypress={search_unique_cols_vehicles.clone().reform(|_| {"price".to_string()})} oninput={ Callback::from({ let
    //                 vehicle_state=vehicle_state.clone(); move |e: InputEvent| { let
    //                 input=e.target_dyn_into::<web_sys::HtmlInputElement>().unwrap();
    //             vehicle_state.set((vehicle_state.0.clone(), vehicle_state.1.clone(), vehicle_state.2.clone(), input.value(),
    //             vehicle_state.4.clone(), vehicle_state.5));
    //             }
    //             })}
    //             class="border rounded px-4 py-2 mr-2"
    //             />
    //             <input list="data" placeholder="Data" value={vehicle_state.4.clone()} onfocus={search_unique_cols_vehicles.clone().reform(|_| {"data".to_string()})} onkeypress={search_unique_cols_vehicles.clone().reform(|_| {"data".to_string()})} oninput={ Callback::from({ let
    //                 vehicle_state=vehicle_state.clone(); move |e: InputEvent| { let
    //                 input=e.target_dyn_into::<web_sys::HtmlInputElement>().unwrap();
    //             vehicle_state.set((vehicle_state.0.clone(), vehicle_state.1.clone(), vehicle_state.2.clone(),
    //             vehicle_state.3.clone(), input.value(), vehicle_state.5));
    //             }
    //             })}
    //             class="border rounded px-4 py-2 mr-2"
    //             />
    //             <button onclick={ if vehicle_state.5.is_some() {update_vehicle.clone()} else {create_vehicle.clone()}}
    //                 class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded">
    //                 { if vehicle_state.5.is_some() {"Update Vehicle"} else {"Create Vehicle"}}
    //             </button>
    //             if !message.is_empty() {
    //             <p class="text-green-500 mt-2">{&*message}</p>
    //             }
    //         </div>
    //         <button onclick={get_vehicles.reform(|_| {})}
    //             class="bg-gray-500 hovering:bg-gray-700 text-white font-bold py-2 px-4 rounded">
    //             { "Fetch Vehicle List" }
    //         </button>
    //         <img src={format!("data:image/png;base64,{}", qr_img.clone().to_string())}/>


    //         <h2 class="text-2x1 font-bold text-gray-700 mb-2"> { "Vehicle List" } </h2>

    //         <ul class="list-disc pl-5">
    //             {for (*vehicles).iter().map(|vehicle| {
    //             let vehicle_id = vehicle.id;
    //             html!(
    //             <li class="mb-2">
    //                 <span class="font-semibold">{ format!("ID: {}, Vehicle Type: {}, Manufacturer: {}, Model: {}, Price: {},
    //                     Data: {}", vehicle.id, vehicle.vehicle_type, vehicle.manufacturer, vehicle.model, vehicle.price,
    //                     vehicle.data)}</span>
    //                 <button onclick={delete_vehicle.clone().reform(move |_| vehicle_id)}
    //                     class="ml-4 bg-red-500 hover:bg-red-700 text-white font-bold py-1 px-2 rounded">
    //                     {"Delete"}
    //                 </button>
    //                 <button onclick={edit_vehicle.clone().reform(move |_| vehicle_id)}
    //                     class="ml-4 bg-yellow-500 hover:bg-yellow-700 text-white font-bold py-1 px-2 rounded">
    //                     {"Edit"}
    //                 </button>
    //                 <button onclick={show_vehicle_qr.clone().reform(move |_| vehicle_id)}
    //                     class="ml-4 bg-green-500 hover:bg-green-700 text-white font-bold py-1 px-2 rounded">
    //                     {"Generate QR"}
    //                 </button>
    //             </li>
    //             )
    //             })}
    //         </ul>
    //     </div>

    // </div>
    )
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Vehicle {
    id: i32,
    vehicle_type: String,
    manufacturer: String,
    model: String,
    price: String,
    data: String,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: String,
}

#[function_component(Info)]
fn main_app(props: &Props) -> Html {
    let Props { id } = props;
    let id = use_state(|| id.to_string());
    let vehicle = use_state_eq(|| Vehicle {
        id: -1,
        data: "".to_string(),
        vehicle_type: "".to_string(),
        manufacturer: "".to_string(),
        model: "".to_string(),
        price: "".to_string(),
    });

    let get_cur_vehicle = {
        let vehicle = vehicle.clone();
        let id = id.clone();

        Callback::from(move |_| {
            let vehicle = vehicle.clone();
            let id = id.clone();

            spawn_local(async move {
                match Request::get(&format!("http://{}:{}/api/vehicles/{}", IP_ADDR, PORT, *id))
                    .send()
                    .await
                {
                    Ok(resp) if resp.ok() => {
                        let fetched_vehicle: Vehicle = resp.json().await.unwrap();
                        vehicle.set(fetched_vehicle);
                    }
                    _ => {}
                }
            });
        })
    };

    let display_vehicle_data = move || -> Html {
        let get_cur_vehicle = get_cur_vehicle.clone();
        get_cur_vehicle.emit(());
        let vehicle = (*vehicle).clone();
        // html!(format!(
        //     "ID: {}, Vehicle Type: {}, Manufacturer: {}, Model: {}, Price: {}, Data: {}",
        //     vehicle.id,
        //     vehicle.vehicle_type,
        //     vehicle.manufacturer,
        //     vehicle.model,
        //     vehicle.price,
        //     vehicle.data
        // ))
        html!(
        <div class="container mx-auto">
            <h1 class="text-2xl font-bold text-center mb-6">{"Vehicle Information"}</h1>
            
            <div class="bg-white shadow-md rounded-lg p-6">
                <table class="min-w-full">
                    <tbody>
                        <tr class="border-b">
                            <td class="py-3 px-4 font-semibold">{"ID:"}</td>
                            <td class="py-3 px-4">{vehicle.id}</td>
                        </tr>
                        <tr class="border-b">
                            <td class="py-3 px-4 font-semibold">{"Vehicle Type:"}</td>
                            <td class="py-3 px-4">{vehicle.vehicle_type}</td>
                        </tr>
                        <tr class="border-b">
                            <td class="py-3 px-4 font-semibold">{"Manufacturer:"}</td>
                            <td class="py-3 px-4">{vehicle.manufacturer}</td>
                        </tr>
                        <tr class="border-b">
                            <td class="py-3 px-4 font-semibold">{"Model:"}</td>
                            <td class="py-3 px-4">{vehicle.model}</td>
                        </tr>
                        <tr class="border-b">
                            <td class="py-3 px-4 font-semibold">{"Price:"}</td>
                            <td class="py-3 px-4">{vehicle.price}</td>
                        </tr>
                        <tr class="border-b">
                            <td class="py-3 px-4 font-semibold">{"Data:"}</td>
                            <td class="py-3 px-4">{vehicle.data}</td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
        )
    
    };

    html!(
        <div>
            <span class="font-semibold">{display_vehicle_data()}</span>
        </div>
    )
}

fn switch(route: Route) -> Html {
    match route {
        Route::Home => html!(<App/>),
        Route::Info { id } => html! (<Info id={id}/>),
    }
}

#[function_component(Main)]
fn app() -> Html {
    html! (
        <BrowserRouter>
            <Switch<Route> render={switch} /> // <- must be child of <BrowserRouter>
        </BrowserRouter>
    )
}

fn main() {
    yew::Renderer::<Main>::new().render();
}
