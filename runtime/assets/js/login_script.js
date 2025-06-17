const form = document.querySelector('.form');
const inputs = form.querySelectorAll('input');
const submitButton = form.querySelector('.button');
const serverErrorDiv = document.getElementById('serverError');
const serverErrorMessage = document.getElementById('serverErrorMessage');

function showError(input, message) {
    const group = input.closest('.form-group');
    const errorElement = group.querySelector('.error-message');

    group.classList.add('error');
    group.classList.remove('valid');
    errorElement.textContent = message;
    errorElement.style.display = 'flex';
    input.setAttribute('aria-invalid', 'true');
}

function showSuccess(input) {
    const group = input.closest('.form-group');
    const errorElement = group.querySelector('.error-message');

    group.classList.remove('error');
    group.classList.add('valid');
    errorElement.style.display = 'none';
    errorElement.textContent = '';
    input.setAttribute('aria-invalid', 'false');
}

function clearValidation(input) {
    const group = input.closest('.form-group');
    const errorElement = group.querySelector('.error-message');

    group.classList.remove('error', 'valid');
    errorElement.style.display = 'none';
    errorElement.textContent = '';
    input.setAttribute('aria-invalid', 'false');
}


function showServerError(message) {
    serverErrorMessage.textContent = message;
    serverErrorDiv.classList.add('show');
    serverErrorDiv.scrollIntoView({ behavior: 'smooth', block: 'center' });
}

function hideServerError() {
    serverErrorDiv.classList.remove('show');
    serverErrorMessage.textContent = '';
}