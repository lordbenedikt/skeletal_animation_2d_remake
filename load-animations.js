  export function uploadFileToLocalStorage() {
    //new
    document.getElementById('files').click();
  }
  function dateiauswahl(evt) {
console.log("auswahl");
    var dateien = evt.target.files; // FileList object
    // Auslesen der gespeicherten Dateien durch Schleife
    for (var i = 0, f; f = dateien[i]; i++) {
      // // nur .Text-Dateien
      // if (!f.na.match('txt.*')) {
      // 	continue;
      // }
      var reader = new FileReader();
      reader.onload = (function (theFile) {
        return function (e) {
          fetch(e.target.result)
            .then(function (response) {
              response.text().then(function (text) {
                window.localStorage.setItem("loaded_anim", text);
                window.localStorage.setItem("loaded_anim", parseInt(window.localStorage.getItem(loaded_anim)) + 1);
              });
            });
        };
      })(f);
      // Als Data URL auslesen.
      reader.readAsDataURL(f);
    }
  }
  
  // Auf neue Auswahl reagieren und gegebenenfalls Funktion dateiauswahl neu ausführen.
  let element = document.getElementById('files');

  element.addEventListener('change', dateiauswahl, false);