import colors from 'vuetify/es5/util/colors'
const path = require("path");
const webpack = require("webpack");


export default {
  // Disable server-side rendering: https://go.nuxtjs.dev/ssr-mode
  ssr: false,

  // Target: https://go.nuxtjs.dev/config-target
  target: 'static',

  // Global page headers: https://go.nuxtjs.dev/config-head
  head: {
    titleTemplate: '%s - swap_frontend',
    title: 'MetaMask Demo',
    htmlAttrs: {
      lang: 'en',
    },
    meta: [
      { charset: 'utf-8' },
      { name: 'viewport', content: 'width=device-width, initial-scale=1' },
      { hid: 'description', name: 'description', content: '' },
      { name: 'format-detection', content: 'telephone=no' },
    ],
    link: [{ rel: 'icon', type: 'image/x-icon', href: '/favicon.ico' }],
  },

  // Global CSS: https://go.nuxtjs.dev/config-css
  css: [],

  // Plugins to run before rendering page: https://go.nuxtjs.dev/config-plugins
  plugins: [],

  // Auto import components: https://go.nuxtjs.dev/config-components
  components: true,

  // Modules for dev and build (recommended): https://go.nuxtjs.dev/config-modules
  buildModules: [
    // https://go.nuxtjs.dev/eslint
    '@nuxtjs/eslint-module',
    // https://go.nuxtjs.dev/vuetify
    '@nuxtjs/vuetify',
  ],

  // Modules: https://go.nuxtjs.dev/config-modules
  modules: [],

  // Vuetify module configuration: https://go.nuxtjs.dev/config-vuetify
  vuetify: {
    customVariables: ['~/assets/variables.scss'],
    theme: {
      dark: true,
      themes: {
        dark: {
          primary: colors.blue.darken2,
          accent: colors.grey.darken3,
          secondary: colors.amber.darken3,
          info: colors.teal.lighten1,
          warning: colors.amber.base,
          error: colors.deepOrange.accent4,
          success: colors.green.accent3,
        },
      },
    },
  },

  // Build Configuration: https://go.nuxtjs.dev/config-build
  build: {

    extend (config) {

      function initCanisterEnv() {
        let localCanisters, prodCanisters;
        try {
          localCanisters = require(path.resolve(
            ".dfx",
            "local",
            "canister_ids.json"
          ));
        } catch (error) {
          console.log("No local canister_ids.json found. Continuing production");
        }
        try {
          prodCanisters = require(path.resolve("canister_ids.json"));
        } catch (error) {
          console.log("No production canister_ids.json found. Continuing with local");
        }
      
        const network =
          process.env.DFX_NETWORK ||
          (process.env.NODE_ENV === "production" ? "ic" : "local");
      
        const canisterConfig = network === "local" ? localCanisters : prodCanisters;
      
        return Object.entries(canisterConfig).reduce((prev, current) => {
          const [canisterName, canisterDetails] = current;
          prev[canisterName.toUpperCase() + "_CANISTER_ID"] =
            canisterDetails[network];
          return prev;
        }, {});
      }


      const canisterEnvVariables = initCanisterEnv();
      const plugins = [
        new webpack.EnvironmentPlugin({
          NODE_ENV: "development",
          ...canisterEnvVariables,
        }),
        new webpack.ProvidePlugin({
          Buffer: [require.resolve("buffer/"), "Buffer"],
          process: require.resolve("process/browser"),
        }),
      ];
      config.plugins.concat(plugins); 
    }
  },
}
