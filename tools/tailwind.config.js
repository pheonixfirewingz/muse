
module.exports = {
    content: [
        './templates/**/*.{jinja}',
        './static/**/*.{css,js}',
        '!./static/tailwind.css',
    ],
    theme: {
        extend: {},
    },
    plugins: [],
};