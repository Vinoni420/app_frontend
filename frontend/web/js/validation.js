/**
 * validates email
 * @param {string} email 
 * @returns {boolean} 
 */
export function validateEmail(email) {
  const re = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return re.test(String(email).toLowerCase());
}

/**
 * validates password
 * @param {string} password
 * @returns {boolean}
 */
export function validatePassword(password) {
  return password && password.length >= 6;
}

/**
 * validates name
 * @param {string} name
 * @returns {boolean}
 */
export function validateFullName(name) {
  if (!name) return false;
  const parts = name.trim().split(' ').filter(part => part.length > 0);
  return parts.length === 2;
}


/**
 * Displays a field-specific error message.
 * @param {string} fieldName - The name of the form field.
 * @param {string} message - The error message to display.
 */
export function showFieldError(fieldName, message) {
  const errorElement = document.getElementById(`${fieldName}-error`);
  if (errorElement) {
    errorElement.textContent = message;
    errorElement.style.display = 'block';
  }
}

/**
 * Clears all field-specific error messages.
 */
export function clearAllFieldErrors() {
  const errorElements = document.querySelectorAll('.error-message');
  errorElements.forEach(el => {
    el.textContent = '';
    el.style.display = 'none';
  });
}