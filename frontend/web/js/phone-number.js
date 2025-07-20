// A variable to store the phone number after initial submission
let validatedPhoneNumber = '';

const phoneForm = document.getElementById('phone-form');
const codeForm = document.getElementById('code-form');
const phoneInput = document.getElementById('phone');
const phoneErrorMsg = document.getElementById('error-msg');
const codeErrorMsg = document.getElementById('code-error-msg');
const resendBtn = document.getElementById('resend-code-btn');
const resendErrorMsg = document.getElementById('resend-error-msg');

// --- Helper Functions (Unchanged) ---

// format phone number to +972 format
function formatPhoneNumber(input) {
  let cleaned = input.trim().replace(/\s+/g, '');

  if (cleaned.startsWith('+972')) {
    cleaned = cleaned.slice(4);
  } else if (cleaned.startsWith('972')) {
    cleaned = cleaned.slice(3);
  } else if (cleaned.startsWith('0')) {
    cleaned = cleaned.slice(1);
  }

  return '+972' + cleaned;
}

// validate Israeli phone number
function isValidPhoneNumber(number) {
  return /^[5][0-9]{8}$/.test(number);
}


// --- REFACTORED AND REUSABLE SMS SENDING LOGIC ---

/**
 * Handles sending the SMS request to the backend.
 * This function is now used by both the initial submission and the "Resend" button.
 * @param {HTMLElement} errorElement - The DOM element where feedback messages should be displayed.
 * @param {boolean} isFirstSend - True if this is the initial click, false if it's a resend.
 */
async function handleSendSmsRequest(errorElement, isFirstSend = false) {
  const userUuid = localStorage.getItem('user_uuid');
  if (!userUuid) {
    window.location.href = './sign-up.html';
    return;
  }

  errorElement.textContent = '...שולח'; // Show immediate feedback

  try {
    const response = await fetch('http://localhost:3000/auth/sign-up/send-sms', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        uuid: userUuid,
        phone_num: validatedPhoneNumber, // Use the stored phone number
      }),
    });

    const data = await response.json();

    if (!response.ok || data?.error_code) {
      // Handle all known errors from the backend
      switch (data.error_code) {
        case 'session_not_found':
          errorElement.textContent = 'לא נמצאה הרשמה. אנא התחילו מההתחלה.';
          localStorage.removeItem('user_uuid');
          window.location.href = './sign-up.html';
          break;
        case 'sms_already_verified':
          errorElement.textContent = 'מספר זה כבר בשימוש';
          break;
        case 'phone_num_not_matching':
          errorElement.textContent = 'מספר טלפון לא תואם לחשבון שנרשם קודם.';
          break;
        case 'need_to_wait_before_resend':
          errorElement.textContent = 'יש להמתין לפני שליחת קוד נוסף.';
          break;
        case 'invalid_number':
          errorElement.textContent = 'מספר לא תקין או לא ניתן לשליחה.';
          break;
        case 'api_error':
          errorElement.textContent = 'שגיאה עם ספק השליחה. נסה שוב מאוחר יותר.';
          break;
        case 'InternalError':
        default:
          errorElement.textContent = 'שגיאה פנימית. נסה שוב מאוחר יותר.';
          break;
      }
      return;
    }

    // --- SMS sent successfully ---
    if (isFirstSend) {
      phoneForm.classList.add('hidden');
      codeForm.classList.remove('hidden');
    } else {
      errorElement.textContent = 'קוד חדש נשלח בהצלחה!';
    }

  } catch (err) {
    errorElement.textContent = 'שגיאת רשת. נסה שוב מאוחר יותר.';
  }
}


// --- Event Listener 1: Handle INITIAL phone form submission ---
phoneForm.addEventListener('submit', async (e) => {
  e.preventDefault();
  phoneErrorMsg.textContent = '';

  const rawInput = phoneInput.value;
  const formattedPhone = formatPhoneNumber(rawInput);
  const strippedPhone = formattedPhone.replace('+972', '');

  if (!isValidPhoneNumber(strippedPhone)) {
    phoneErrorMsg.textContent = 'אנא הזינו מספר טלפון תקין';
    return;
  }

  // Store the validated number for future use (by the resend button)
  validatedPhoneNumber = formattedPhone;

  // Call the reusable function for the first time
  await handleSendSmsRequest(phoneErrorMsg, true);
});


// --- Event Listener 2: Handle RESEND code button click ---
resendBtn.addEventListener('click', async () => {
  // Disable button to prevent spamming
  resendBtn.disabled = true;
  resendErrorMsg.textContent = ''; // Clear previous messages

  // Call the reusable function, targeting the specific error message element for this button
  await handleSendSmsRequest(resendErrorMsg, false);
  
  // Re-enable the button after the request is complete, regardless of outcome
  resendBtn.disabled = false;
});


// --- Event Listener 3: Handle SMS code form submission (Unchanged) ---
codeForm.addEventListener('submit', async (e) => {
    e.preventDefault();

    const successMsg = document.getElementById('success-msg');
    const codeInputs = document.querySelectorAll('.code-box');

    // Clear previous errors and hide success message
    codeErrorMsg.textContent = '';
    successMsg.style.display = 'none';

    // Collect code from the 6 input boxes
    const code = Array.from(codeInputs).map(input => input.value).join('').trim();

    // --- Basic client-side validation ---
    if (code.length !== 6 || !/^\d{6}$/.test(code)) {
      codeErrorMsg.textContent = 'הקוד חייב להכיל בדיוק 6 ספרות.';
      return;
    }

    const userUuid = localStorage.getItem('user_uuid');
    if (!userUuid) {
      window.location.href = './sign-up.html';
      return;
    }

    try {
      const verifyRes = await fetch('http://localhost:3000/auth/sign-up/verify-sms', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ uuid: userUuid, code }),
      });

      const verifyData = await verifyRes.json();

      if (!verifyRes.ok || verifyData.error_code) {
        console.log(verifyData);
        switch (verifyData.error_code) {
          case 'wrong_code':
            codeErrorMsg.textContent = 'קוד שגוי. אנא נסו שוב.';
            codeInputs.forEach(input => input.value = '');
            codeInputs[0].focus();
            break;
          case 'too_many_attempts':
            codeErrorMsg.textContent = 'ניסיונות רבים מדי. אנא בקשו קוד חדש.';
            break;
          case 'need_to_resend_code':
            codeErrorMsg.textContent = 'הקוד פג תוקף. אנא בקשו קוד חדש.';
            break;
          case 'session_not_found':
            alert('החיבור פג תוקף. תועברו להתחלת תהליך ההרשמה.');
            window.location.href = './sign_up.html';
            break;
          case 'InternalError':
          default:
            codeErrorMsg.textContent = 'שגיאת שרת. אנא נסו שוב מאוחר יותר.';
            break;
        }
        return;
      }

      // --- If verification was successful, proceed to complete the sign-up ---
      const completeRes = await fetch('http://localhost:3000/auth/sign-up/complete', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ uuid: userUuid }),
      });

      const completeData = await completeRes.json();

      if (completeData.error_code) {
        codeErrorMsg.textContent = 'ההרשמה נכשלה: ' + completeData.error_code.replace(/_/g, ' ');
        return;
      }

      // Store tokens and user data
      localStorage.setItem('jwt_token', completeData.jwt_token);
      localStorage.setItem('user_data', JSON.stringify(completeData.user_data));

      // Redirect to the dashboard on final success
      window.location.href = './dashboard.html';

    } catch (err) {
      console.error("Network or unexpected error:", err);
      codeErrorMsg.textContent = 'שגיאת רשת. אנא בדקו את החיבור ונסו שוב.';
    }
});