/**
 * @see https://prettier.io/docs/en/configuration.html
 * @type {import("prettier").Config}
 */
const config = {
    plugins: [],
    tabWidth: 4,
    printWidth: 85,
    useTabs: false,
    trailingComma: "none",
    semi: true,
    singleQuote: false,
    overrides: [
        {
            files: ["*.json", "*.yaml", "*.yml"],
            options: {
                tabWidth: 2
            }
        }
    ]
};

export default config;
