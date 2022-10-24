  var loaded_anim = "nothing";

  export function uploadFile() {
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
                loaded_anim = text;
                done();
              });
            });
        };
      })(f);
      // Als Data URL auslesen.
      reader.readAsDataURL(f);
    }
  }
  export function done() {
    console.log(loaded_anim);
  }
  export function test() {
    console.log("test");
  }
  // Auf neue Auswahl reagieren und gegebenenfalls Funktion dateiauswahl neu ausführen.
  let element = document.getElementById('files');

  element.addEventListener('change', dateiauswahl, false);