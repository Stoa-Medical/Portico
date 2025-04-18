/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./src/**/*.{html,js,svelte,ts}",
    "./node_modules/flowbite-svelte/**/*.{html,js,svelte,ts}",
  ],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        ink: {
          DEFAULT: "#262431",
          mid: "#B2B1B9",
          light: "#F3F2F4",
        },
        papyrus: {
          DEFAULT: "#ECD8A8",
          mid: "#F4E9CE",
          light: "#F8F2E3",
        },
        sky: {
          DEFAULT: "#B0C3E8",
          mid: "#DDE6F8",
          light: "#EAEFF9",
        },
        sea: {
          DEFAULT: "#A7E7DF",
          mid: "#C9F4EF",
          light: "#ECF6F5",
        },
        error: {
          DEFAULT: "#E58382",
          mid: "#F1B2B0",
          light: "#FDEDED",
        },
        success: {
          DEFAULT: "#76D5AC",
          mid: "#A2EACB",
          light: "#E3F7EF",
        },
        warning: {
          DEFAULT: "#EEBD8E",
          mid: "#F8DABE",
          light: "#FFF4EA",
        },
        white: "#FDFCFB",
      },
      fontFamily: {
        primary: ['"Helvetica"', "sans-serif"],
        secondary: ['"Cormorant Garamond"', "serif"],
      },
    },
  },
  plugins: [require("flowbite/plugin")],
  darkMode: "class",
};
