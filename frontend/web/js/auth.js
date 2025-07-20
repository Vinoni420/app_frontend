const API_BASE = "http://localhost:3000";

/**
 * Starts the password-based sign-up process.
 * @param {string} email - The user's email.
 * @param {string} password - The user's password.
 * @param {string} name - The user's full name.
 * @param {string} captchaToken - The reCAPTCHA token.
 * @returns {Promise<{sign_up_token: string}>} - A promise that resolves with the sign-up token.
 * @throws {Error} - Throws an error with a specific error code if the request fails.
 */
export async function startPasswordSignUp(email, password, name, captchaToken) {
  const payload = {
    method: "p_a_s_s_w_o_r_d",
    email,
    password,
    name,
    captcha_token: captchaToken,
  };

  const response = await fetch(`${API_BASE}/auth/sign-up/start`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(payload),
  });

  const data = await response.json();

  return data;
}