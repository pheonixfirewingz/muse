import { heroui } from "@heroui/theme";

/** @type {import('tailwindcss').Config} */
export default {
  content: [
    './index.html',
    './src/**/*.{js,ts,jsx,tsx}',
  ],
  theme: {
    extend: {},
  },
  darkMode: "class",
  plugins: [
    heroui({
  "themes": {
    "light": {
      "colors": {
        "default": {
          "50": "#fafafa",
          "100": "#f2f2f3",
          "200": "#ebebec",
          "300": "#e3e3e6",
          "400": "#dcdcdf",
          "500": "#d4d4d8",
          "600": "#afafb2",
          "700": "#8a8a8c",
          "800": "#656567",
          "900": "#404041",
          "foreground": "#000",
          "DEFAULT": "#d4d4d8"
        },
        "primary": {
          "50": "#f5e1e4",
          "100": "#e6b6bd",
          "200": "#d78c97",
          "300": "#c86171",
          "400": "#ba374a",
          "500": "#ab0c24",
          "600": "#8d0a1e",
          "700": "#6f0817",
          "800": "#510611",
          "900": "#33040b",
          "foreground": "#fff",
          "DEFAULT": "#ab0c24"
        },
        "secondary": {
          "50": "#eddfdf",
          "100": "#d3b3b3",
          "200": "#ba8686",
          "300": "#a05959",
          "400": "#872d2d",
          "500": "#6d0000",
          "600": "#5a0000",
          "700": "#470000",
          "800": "#340000",
          "900": "#210000",
          "foreground": "#fff",
          "DEFAULT": "#6d0000"
        },
        "success": {
          "50": "#e2f8ec",
          "100": "#b9efd1",
          "200": "#91e5b5",
          "300": "#68dc9a",
          "400": "#40d27f",
          "500": "#17c964",
          "600": "#13a653",
          "700": "#0f8341",
          "800": "#0b5f30",
          "900": "#073c1e",
          "foreground": "#000",
          "DEFAULT": "#17c964"
        },
        "warning": {
          "50": "#fef4e4",
          "100": "#fce4bd",
          "200": "#fad497",
          "300": "#f9c571",
          "400": "#f7b54a",
          "500": "#f5a524",
          "600": "#ca881e",
          "700": "#9f6b17",
          "800": "#744e11",
          "900": "#4a320b",
          "foreground": "#000",
          "DEFAULT": "#f5a524"
        },
        "danger": {
          "50": "#fde1e1",
          "100": "#fbb8b8",
          "200": "#f98e8e",
          "300": "#f76464",
          "400": "#f43b3b",
          "500": "#f21111",
          "600": "#c80e0e",
          "700": "#9d0b0b",
          "800": "#730808",
          "900": "#490505",
          "foreground": "#000",
          "DEFAULT": "#f21111"
        },
        "background": "#ffffff",
        "foreground": "#000000",
        "content1": {
          "DEFAULT": "#ffffff",
          "foreground": "#000"
        },
        "content2": {
          "DEFAULT": "#f4f4f5",
          "foreground": "#000"
        },
        "content3": {
          "DEFAULT": "#e4e4e7",
          "foreground": "#000"
        },
        "content4": {
          "DEFAULT": "#d4d4d8",
          "foreground": "#000"
        },
        "focus": "#006FEE",
        "overlay": "#000000"
      }
    },
    "dark": {
      "colors": {
        "default": {
          "50": "#1c1c1c",
          "100": "#383838",
          "200": "#545454",
          "300": "#707070",
          "400": "#8c8c8c",
          "500": "#a3a3a3",
          "600": "#bababa",
          "700": "#d1d1d1",
          "800": "#e8e8e8",
          "900": "#ffffff",
          "foreground": "#000",
          "DEFAULT": "#8c8c8c"
        },
        "primary": {
          "50": "#33040b",
          "100": "#510611",
          "200": "#6f0817",
          "300": "#8d0a1e",
          "400": "#ab0c24",
          "500": "#ba374a",
          "600": "#c86171",
          "700": "#d78c97",
          "800": "#e6b6bd",
          "900": "#f5e1e4",
          "foreground": "#fff",
          "DEFAULT": "#ab0c24"
        },
        "secondary": {
          "50": "#210000",
          "100": "#340000",
          "200": "#470000",
          "300": "#5a0000",
          "400": "#6d0000",
          "500": "#872d2d",
          "600": "#a05959",
          "700": "#ba8686",
          "800": "#d3b3b3",
          "900": "#eddfdf",
          "foreground": "#fff",
          "DEFAULT": "#6d0000"
        },
        "success": {
          "50": "#073c1e",
          "100": "#0b5f30",
          "200": "#0f8341",
          "300": "#13a653",
          "400": "#17c964",
          "500": "#40d27f",
          "600": "#68dc9a",
          "700": "#91e5b5",
          "800": "#b9efd1",
          "900": "#e2f8ec",
          "foreground": "#000",
          "DEFAULT": "#17c964"
        },
        "warning": {
          "50": "#4a320b",
          "100": "#744e11",
          "200": "#9f6b17",
          "300": "#ca881e",
          "400": "#f5a524",
          "500": "#f7b54a",
          "600": "#f9c571",
          "700": "#fad497",
          "800": "#fce4bd",
          "900": "#fef4e4",
          "foreground": "#000",
          "DEFAULT": "#f5a524"
        },
        "danger": {
          "50": "#490505",
          "100": "#730808",
          "200": "#9d0b0b",
          "300": "#c80e0e",
          "400": "#f21111",
          "500": "#f43b3b",
          "600": "#f76464",
          "700": "#f98e8e",
          "800": "#fbb8b8",
          "900": "#fde1e1",
          "foreground": "#000",
          "DEFAULT": "#f21111"
        },
        "background": "#000000",
        "foreground": "#ffffff",
        "content1": {
          "DEFAULT": "#18181b",
          "foreground": "#fff"
        },
        "content2": {
          "DEFAULT": "#27272a",
          "foreground": "#fff"
        },
        "content3": {
          "DEFAULT": "#3f3f46",
          "foreground": "#fff"
        },
        "content4": {
          "DEFAULT": "#52525b",
          "foreground": "#fff"
        },
        "focus": "#006FEE",
        "overlay": "#ffffff"
      }
    }
  },
  "layout": {
    "disabledOpacity": "0.6"
  }
}),
  ]
};
