const { invoke } = window.__TAURI__.tauri;

let games_list = {};

let button_text = {
  "install": "Install game!",
  "installing": "Installing...",
  "play": "Play game!"
};

// Prevents the game selector from being interacted with 
// while a game is installing.
let lockout_selector = false;

async function action_game() {
  let gamesListEl = document.querySelector("#games-list");
  let actionButtonEl = document.querySelector("#action-game");
  let pkgname = gamesListEl[gamesListEl.selectedIndex].value;

  if (!games_list[pkgname].installed) { // Install
    // Prevent selector and action button from being clicked on 
    // and use the action button as a status message.
    lockout_selector = true;
    actionButtonEl.disabled = true;
    actionButtonEl.textContent = button_text["installing"];
    
    // Trigger game install. The Rust code for this spawns a new
    // thread which, when complete, will cause the promise 
    // returned by invoke to complete, triggering the then call.
    invoke("install_game", { pkgname: pkgname }).then(() => {
      // Manually set installed in the games list so that the rest
      // of the UI code works properly without reloading the full
      // games list.
      games_list[pkgname].installed = true;

      // Re-enable UI
      lockout_selector = false;
      actionButtonEl.disabled = false;

      // Since the selected game has now been installed, this button
      // will trigger play instead of install.
      actionButtonEl.textContent = button_text["play"];
    });
  } else {
    // This includes a .exec call so this application closes
    // at this point.
    await invoke("run_game", { pkgname: pkgname });
  }
}

async function load_games_list() {
  let games_list_all = await invoke("get_all_games");
  let games_list_installed = await invoke("get_installed_games");

  // Convert input lists to a dict containing the relevant information
  // and whether the game is installed or not.
  for (let i = 0; i < games_list_all.length; i++) {
    let name = games_list_all[i].pkgname;
    games_list[name] = games_list_all[i];
    games_list[name].installed = false;
  }

  for (let i = 0; i < games_list_installed.length; i++) {
    let name = games_list_installed[i].pkgname;
    games_list[name].installed = true;
  }

  // Populate select with the available games.
  let gamesListEl = document.querySelector("#games-list");

  for (let name in games_list) {
    let node = document.createElement("option");
    node.textContent = games_list[name].fullname;
    node.value = games_list[name].pkgname;
    gamesListEl.add(node);
  }
}

async function game_list_clicked() {
  if (lockout_selector) {
    return;
  }

  let gamesListEl = document.querySelector("#games-list");
  let pkgname = gamesListEl[gamesListEl.selectedIndex].value;

  let installButtonEl = document.querySelector("#action-game");
  // There is no selected element before this is clicked so the
  // page loads with this button disabled. 
  installButtonEl.disabled = false;

  let descriptionEl = document.querySelector("#description");

  // Update button state and description with selected game.
  if (games_list.length != 0) {
    if (games_list[pkgname].installed) {
      installButtonEl.textContent = button_text["play"];
    } else {
      installButtonEl.textContent = button_text["install"];
    }

    descriptionEl.textContent = games_list[pkgname].description;
  } 
}

async function games_list_mousedown(e) {
  // Prevents clicking on the select element from changing the
  // selected option.
  if (lockout_selector) {
    e.preventDefault();
  }
}

window.addEventListener("DOMContentLoaded", () => {
  document
    .querySelector("#action-game")
    .addEventListener("click", () => action_game());

  let gamesListEl = document
    .querySelector("#games-list");
  gamesListEl.addEventListener("click", () => game_list_clicked());
  gamesListEl.addEventListener("mousedown", (e) => games_list_mousedown(e));

  load_games_list();
});
