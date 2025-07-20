import { startPasswordSignUp } from './auth.js';
import {
  validateEmail,
  validatePassword,
  validateFullName,
  showFieldError,
  clearAllFieldErrors,
} from './validation.js';

document.addEventListener("DOMContentLoaded", () => {
  const form = document.getElementById("signup-form");
  if (!form) return;

  const serverErrorDiv = document.getElementById("server-error-container");
  const submitBtn = form.querySelector("button[type=submit]");

  form.addEventListener("submit", async (e) => {
    e.preventDefault();
clearAllFieldErrors();
if (serverErrorDiv) serverErrorDiv.textContent = '';

const email = form.email.value.trim();
const password = form.password.value;
const name = form.name.value.trim();
let hasErrors = false;
let firstErrorField = null;

// --- Email Validation ---
if (!email) {
  showFieldError("email", "אנא הזינו אימייל");
  if (!firstErrorField) firstErrorField = "email";
  hasErrors = true;
} else if (!validateEmail(email)) {
  showFieldError("email", "אנא הזינו כתובת אימייל תקנית.");
  if (!firstErrorField) firstErrorField = "email";
  hasErrors = true;
}

// --- Password Validation ---
if (!password) {
  showFieldError("password", "אנא הזינו סיסמה.");
  if (!firstErrorField) firstErrorField = "password";
  hasErrors = true;
} else if (!validatePassword(password)) {
  showFieldError("password", "הסיסמה חייבת להכיל לפחות 6 תווים.");
  if (!firstErrorField) firstErrorField = "password";
  hasErrors = true;
}

// --- Name Validation ---
if (!name) {
  showFieldError("name", "אנא הזינו שם מלא");
  if (!firstErrorField) firstErrorField = "name";
  hasErrors = true;
} else if (!validateFullName(name)) {
  showFieldError("name", "אנא הזינו שם מלא (שם פרטי ושם משפחה).");
  if (!firstErrorField) firstErrorField = "name";
  hasErrors = true;
}

// --- reCAPTCHA Validation ---
let captchaToken = '';
try {
  captchaToken = grecaptcha.getResponse();
  if (!captchaToken) {
    showFieldError("captcha", "אנא אשרו שאתם לא רובוט.");
    if (!firstErrorField) firstErrorField = "captcha";
    hasErrors = true;
  }
} catch (error) {
  showFieldError("captcha", "טעינת reCAPTCHA נכשלה, נסו לרענן את הדף.");
  if (!firstErrorField) firstErrorField = "captcha";
  hasErrors = true;
}

if (hasErrors) {
  if (firstErrorField) {
    document.getElementById(firstErrorField)?.focus();
  }
  return;
}


    if (hasErrors) return;

    // --- Form Submission ---
    try {
      submitBtn.disabled = true;
      submitBtn.textContent = "טוען...";

      const result = await startPasswordSignUp(email, password, name, captchaToken);
      
      localStorage.setItem('user_uuid', result.sign_up_token);
      window.location.href = './phone-number.html';

      form.reset();
      grecaptcha.reset();

    } catch (err) {
      handleSignUpError(err);
      grecaptcha.reset();
    } finally {
      submitBtn.disabled = false;
      submitBtn.textContent = "הרשמה";
    }
  });

  function handleSignUpError(err) {
      let message = "";
      switch (err.message) {
        case "captcha_verification_failed":
          message = "אימות CAPTCHA נכשל. אנא נסו שוב.";
          break;
        case "email_already_exists":
          showFieldError("email", "אימייל כבר רשום במערכת.");
          return; // Return to avoid showing a generic server error
        case "internal_error":
          message = "אירעה שגיאה פנימית, אנא נסו שוב מאוחר יותר.";
          break;
        default:
          message = "שגיאה לא מוכרת: " + err.message;
      }
      
      if (message && serverErrorDiv) {
        serverErrorDiv.textContent = message;
      }
  }
});