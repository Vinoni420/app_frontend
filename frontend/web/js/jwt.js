const VERIFY_ENDPOINT = '/auth/me';
const BACKEND_URL = "http://localhost:3000"

function getToken() {
  return localStorage.getItem('jwt_token');
}

function removeToken() {
  localStorage.removeItem('jwt_token');
}

async function verifyJwt(token) {
  const res = await fetch(`${BACKEND_URL}${VERIFY_ENDPOINT}`, {
    method: 'GET',
    headers: { Authorization: `Bearer ${token}` },
  });
  if (!res.ok) throw new Error('Invalid token');
  return res.json();
}

(async function () {
  const script = document.currentScript;
  const behavior = script.dataset.behavior;  // protected or public
  const redirect = script.dataset.redirect;

  const token = getToken();

  if (behavior === 'protected') {
    if (!token) return (window.location.href = redirect); // there is no token so go to login

    try {
      const data = await verifyJwt(token); // verify token
      window.currentUser = data.user_data;
    } catch {
      removeToken();
      window.location.href = redirect; // clear expired or invalid token
    }
  }

  if (behavior === 'public') {
    if (!token) return; // there is no token so stay in login

    try {
      const data = await verifyJwt(token);
      if (data?.user_data) {
        window.location.href = redirect;
      }
    } catch(err){
      console.log("deleted: ", err)
      removeToken(); // clear expired or invalid token
    }
  }
})();