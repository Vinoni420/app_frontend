const togglePasswordButton = document.getElementById('toggle-password');
const passwordInput = document.getElementById('password');

togglePasswordButton.addEventListener('click', () => {
  const type = passwordInput.getAttribute('type');
  if (type === 'password') {
    passwordInput.setAttribute('type', 'text');
    togglePasswordButton.innerHTML = '<i class="fa-solid fa-eye-slash"></i>';
    togglePasswordButton.setAttribute('aria-label', 'Hide password');
  } else {
    passwordInput.setAttribute('type', 'password');
    togglePasswordButton.innerHTML = '<i class="fa-solid fa-eye"></i>';
    togglePasswordButton.setAttribute('aria-label', 'Show password');
  }
});