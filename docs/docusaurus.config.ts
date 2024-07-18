import { themes as prismThemes } from 'prism-react-renderer';
import type { Config } from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

const config: Config = {
  title: 'Live Compositor',
  favicon: 'img/favicon.png',

  // Set the production url of your site here
  url: 'https://compositor.live',
  // Set the /<baseUrl>/ pathname under which your site is served
  // For GitHub pages deployment, it is often '/<projectName>/'
  baseUrl: '/',

  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: 'membraneframework', // Usually your GitHub org/user name.
  projectName: 'live_compositor', // Usually your repo name.

  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'throw',
  onBrokenAnchors: 'throw',
  trailingSlash: false,

  // Even if you don't use internationalization, you can use this field to set
  // useful metadata like html lang. For example, if your site is Chinese, you
  // may want to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          path: 'pages',
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
        gtag: {
          trackingID: 'G-SEF91C2QGB',
          anonymizeIP: true,
        },
      } satisfies Preset.Options,
    ],
  ],

  plugins: [
    [
      '@docusaurus/plugin-client-redirects',
      {
        redirects: [
          {
            to: '/docs/intro',
            from: '/docs',
          },
        ],
      },
    ],
  ],

  themeConfig: {
    colorMode: {
      defaultMode: 'dark',
    },
    navbar: {
      logo: {
        alt: 'logo',
        src: 'img/logo.svg',
        srcDark: 'img/logo-dark.svg',
      },
      items: [
        {
          to: '/docs/intro',
          position: 'right',
          className: 'navbar-docs-link',
          'aria-label': 'Docs',
        },
        {
          to: 'https://github.com/membraneframework/live_compositor',
          position: 'right',
          className: 'navbar-github-link',
          'aria-label': 'GitHub repository',
        },
      ],
    },
    footer: {
      style: 'dark',
      copyright: `Copyright © ${new Date().getFullYear()} Software Mansion S.A.`,
    },
    prism: {
      theme: prismThemes.duotoneDark,
      additionalLanguages: ['wgsl', 'http', 'elixir', 'bash'],
    },
    algolia: {
      appId: 'AB30AX8OU1',
      apiKey: '4dae5f71952b8ebd63dd7645128c3b24',
      indexName: 'compositor',
      contextualSearch: false,
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
