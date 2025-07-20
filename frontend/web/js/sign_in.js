// ==========================
// Constants
// ==========================
const LOGIN_ENDPOINT = '/auth/sign-in';
const REDIRECT_AFTER_LOGIN = 'dashboard.html';

// ==========================
// DOM Elements
// ==========================
const form = document.getElementById('signInForm');
const errorMessage = document.getElementById('error-message');

// ==========================
// Token Helpers
// ==========================
function saveToken(token) {
  localStorage.setItem('jwt_token', token);
}

function getToken() {
  return localStorage.getItem('jwt_token');
}

function removeToken() {
  localStorage.removeItem('jwt_token');
}

// ==========================
// Auth: Email & Password
// ==========================
async function login(email, password) {
  const payload = {
    method: 'p_a_s_s_w_o_r_d',
    email,
    password,
  };

  const response = await fetch(`${BACKEND_URL}${LOGIN_ENDPOINT}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  });

  if (!response.ok) {
    throw new Error('Login failed');
  }

  return response.json();
}

// ==========================
// Auth: Google Sign-In
// ==========================
async function handleCredentialResponse(response) {
  const idToken = response.credential;
  console.log(idToken)

  try {
    const backendResponse = await fetch(`${BACKEND_URL}${LOGIN_ENDPOINT}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        method: 'g_o_o_g_l_e',
        id_token: idToken,
      }),
    });

    if (!backendResponse.ok) {
      throw new Error('Google sign-in failed');
    }

    const data = await backendResponse.json();
    saveToken(data.jwt_token);
    window.location.href = REDIRECT_AFTER_LOGIN;

  } catch (err) {
    console.error('Google Sign-In error:', err);
    errorMessage.textContent = 'Google sign-in failed.';
    errorMessage.classList.remove('hidden');
  }
}

// ==========================
// Handlers
// ==========================
async function handleLoginSubmit(event) {
  event.preventDefault();

  const email = form.email.value.trim();
  const password = form.password.value;

  if (!email || !password) {
    errorMessage.textContent = 'Please fill in both email and password.';
    errorMessage.classList.remove('hidden');
    return;
  }

  // Simple email format check
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  if (!emailRegex.test(email)) {
    errorMessage.textContent = 'Please enter a valid email address.';
    errorMessage.classList.remove('hidden');
    return;
  }

  try {
    const data = await login(email, password);
    saveToken(data.jwt_token);
    window.location.href = REDIRECT_AFTER_LOGIN;
  } catch (err) {
    errorMessage.textContent = 'Email or password are incorrect.';
    errorMessage.classList.remove('hidden');
  }
}

// ==========================
// Init
// ==========================
form?.addEventListener('submit', handleLoginSubmit);