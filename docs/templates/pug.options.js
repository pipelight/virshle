// Filter for Zora Tera template engine.
module.exports = {
  doctype: "html",
  filters: {
    tera: function(text, options) {
      return text;
    },
  },
};
