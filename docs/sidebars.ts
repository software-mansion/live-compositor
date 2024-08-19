import type { SidebarsConfig } from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  sidebar: [
    {
      label: 'Getting started',
      type: 'doc',
      id: 'intro',
    },
    {
      type: 'category',
      label: 'Guides',
      link: {
        type: 'generated-index',
      },
      items: [
        {
          type: 'doc',
          id: 'guides/quick-start',
          label: 'Quick start',
        },
        {
          type: 'doc',
          id: 'guides/deliver-input',
          label: 'Deliver input streams',
        },
        {
          type: 'doc',
          id: 'guides/receive-output',
          label: 'Receive output streams',
        },
        {
          type: 'doc',
          id: 'guides/layouts',
          label: 'Layouts',
        },
        {
          type: 'doc',
          id: 'guides/view-transition',
          label: 'Transitions (View/Rescaler)',
        },
      ],
    },
    {
      type: 'category',
      label: 'Concepts',
      link: {
        type: 'doc',
        id: 'concept/overview',
      },
      items: [
        { type: 'ref', id: 'concept/overview', label: 'Overview' },
        'concept/component',
        'concept/layouts',
        'concept/shaders',
        'concept/web',
      ],
    },
    {
      type: 'category',
      label: 'Deployment',
      link: {
        type: 'doc',
        id: 'deployment/overview',
      },
      items: [
        {
          type: 'ref',
          id: 'deployment/overview',
          label: 'Overview',
        },
        {
          type: 'doc',
          id: 'deployment/requirements',
          label: 'Requirements',
        },
        {
          type: 'doc',
          id: 'deployment/configuration',
          label: 'Configuration',
        },
        {
          type: 'doc',
          id: 'deployment/aws-ec2',
          label: 'Example: AWS EC2',
        },
      ],
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
          id: 'api/events',
          label: 'Events',
        },
        {
          type: 'category',
          label: 'Components',
          collapsible: false,
          description: 'Basic blocks used to define a scene.',
          items: [
            {
              type: 'autogenerated',
              dirName: 'api/components',
            },
          ],
        },
        {
          type: 'category',
          label: 'Renderers',
          collapsible: false,
          description: 'Resources that need to be registered first before they can be used.',
          items: ['api/renderers/shader', 'api/renderers/image', 'api/renderers/web'],
        },
        {
          type: 'category',
          label: 'Outputs',
          collapsible: false,
          description: 'Elements that deliver generated media.',
          items: ['api/outputs/rtp', 'api/outputs/mp4'],
        },
        {
          type: 'category',
          label: 'Inputs',
          collapsible: false,
          description: 'Elements that deliver media from external sources.',
          items: ['api/inputs/rtp', 'api/inputs/mp4', 'api/inputs/decklink'],
        },
      ],
    },
  ],
};

export default sidebars;
