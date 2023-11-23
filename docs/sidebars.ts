import type { SidebarsConfig } from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  sidebar: [
    'intro',
    {
      label: 'Get started',
      type: 'category',
      items: ['get-started/elixir', 'get-started/node'],
      collapsed: true,
      link: {
        type: 'doc',
        id: 'get-started',
      },
    },
    {
      type: 'category',
      label: 'API Reference',
      collapsible: false,
      link: {
        type: 'generated-index',
      },
      items: [
        {
          type: 'doc',
          id: 'api/routes',
          label: 'HTTP Routes',
        },
        {
          type: 'doc',
          id: 'api/io',
          label: 'Input/Output streams',
        },
        {
          type: 'category',
          label: 'Components',
          collapsible: false,
          description: 'Basic blocks used to define a scene.',
          items: ['api/components/shader', 'api/components/web'],
        },
        {
          type: 'category',
          label: 'Renderers',
          collapsible: false,
          description: 'Resources that need to be registered first before they can be used.',
          items: ['api/renderers/shader'],
        },
      ],
    },
  ],
};

export default sidebars;