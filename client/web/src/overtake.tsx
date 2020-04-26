export default function overtake(innerHTML: string) {
  // styles defined in index.html stylesheet
  const tempOverflowY = document.documentElement.style.overflowY;
  const modal = document.createElement('div')
  modal.className = 'overtake'
  const modalInner = document.createElement('div')
  modalInner.innerHTML = innerHTML;
  const closeModal = document.createElement('button');
  closeModal.className = 'close';
  closeModal.innerHTML = "X";

  closeModal.addEventListener('click', function closeModal() {
    document.documentElement.style.overflowY = tempOverflowY 
    document.body.removeChild(modal);
  })
  modal.appendChild(closeModal)
  modal.appendChild(modalInner)
  document.documentElement.style.overflowY = 'hidden';
  document.body.appendChild(modal)
}

