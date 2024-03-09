const { invoke } = window.__TAURI__.tauri;
const { open } = window.__TAURI__.dialog;

window.addEventListener("DOMContentLoaded", () => {

  const fileContainer = document.querySelector("#file-container");
  const fileNameSpan = document.querySelector("#fileNameSpan");
  const encodeDiv = document.querySelector("#encodeDiv")
  const radios = document.querySelectorAll(".radio")
  const encodeRadio = radios[0];
  const processButton = document.querySelector("#processButton")
  const messageInput = document.querySelector("#messageToEncode")
  const infoDiv = document.querySelector("#infoDiv")
  const info = document.querySelector("#info")
  let fileName = "";

  const readFile = async () => {
    try {
      const selectedPath = await open({
        multiple: false,
        title: "Wybierz Plik",
      }) 
      fileName = selectedPath
      fileNameSpan.textContent = fileName
    } catch (error) {
      console.log(error) 
    }
  }

  fileContainer.addEventListener("click", () => {
    readFile();
  })

  processButton.addEventListener("click", () => {
    if (encodeRadio.checked){
      if (fileName == "") {
        info.textContent = "Error: Please choose a valid file!"
        return;
      }
      if (messageInput.value == "") {
        info.textContent = "Error: Please enter message!";
        return;
      } 
      invoke("encode", { path: fileName, message: messageInput.value }).then((message) => {
        let startingPoint = message.split(',')[0];
        let endingPoint = message.split(',')[1];
        let messageContent = `Successfully wrote message! <br>
Starting Position: 0x${startingPoint} <br>
Ending Position: 0x${endingPoint} <br>
Content: ${messageInput.value}`
        
        info.innerHTML = messageContent
      })
    } else if (encodeRadio.checked == false){
      if (fileName == "") {
        info.textContent = "Error: Please choose a valid file!"
        return;
      }
      invoke("decode", { path: fileName }).then((message) => {
        info.textContent = message
      })
    } else {
      console.log("error");
    }
  })

  setInterval(() => {
    if (encodeRadio.checked){
      encodeDiv.style.display = "flex"
    } else {
      encodeDiv.style.display = "none"
    }
  }, 100)

});
