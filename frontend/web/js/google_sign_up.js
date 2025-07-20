const GOOGLE_CLIENT_ID = '846963901431-hhi0lbod25gqqkm15tdosjgdj0uv5fjf.apps.googleusercontent.com'; 
const BACKEND_SIGN_UP_URL = 'http://localhost:3000/auth/sign-up/start';
const REDIRECT_ON_SUCCESS_URL = 'dashboard.html';

window.onload = () => {
  if (typeof google === 'undefined' || !google.accounts) {
    console.error('Google Identity Services library not loaded.');
    displayError('שירותי הכניסה של גוגל לא זמינים.');
    return;
  }

  google.accounts.id.initialize({
    client_id: GOOGLE_CLIENT_ID,
    callback: handleCredentialResponse,
  });

  google.accounts.id.renderButton(
    document.getElementById('google-signin-button'),
    {
      theme: 'outline',   
      size: 'large',      
      text: 'signup_with',
      shape: 'rectangular', 
      logo_alignment: 'left'
    }
  );
};

async function handleCredentialResponse(response) {
  const idToken = response.credential;

  try {
    const backendResponse = await fetch(BACKEND_SIGN_UP_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        method: 'g_o_o_g_l_e',
        id_token: idToken, 
      }),
    });

    const data = await backendResponse.json();

    if (!backendResponse.ok) {
      throw new Error(data.message || `Request failed with status ${backendResponse.status}`);
    }

    localStorage.setItem('jwt_token', data.jwt_token);
    window.location.href = REDIRECT_ON_SUCCESS_URL;

  } catch (err) {
    console.error('Google Sign-In error:', err);
    displayError('הכניסה באמצעות גוגל נכשלה. אנא נסה שוב.');
  }
}

function displayError(message) {
  const errorMessageElement = document.getElementById('error-message');
  if (errorMessageElement) {
    errorMessageElement.textContent = message;
    errorMessageElement.style.display = 'block';
  }
}