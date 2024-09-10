import { Api } from 'live-compositor';

export function example02(): Api.Component {
  return {
    type: 'view',
    background_color_rgba: '#ffffffd9',
    children: [
      {
        type: 'tiles',
        width: 1920,
        padding: 15,
        children: [
          {
            type: 'input_stream',
            input_id: 'input_1',
          },
          {
            type: 'image',
            image_id: 'bunny',
          },
          {
            type: 'input_stream',
            input_id: 'input_3',
          },
          {
            type: 'shader',
            shader_id: 'rounded_corners',
            shader_param: {
              type: 'f32',
              value: 64,
            },
            children: [
              {
                type: 'image',
                image_id: 'person',
              },
            ],
            resolution: {
              width: 3000,
              height: 2000,
            },
          },
          {
            type: 'input_stream',
            input_id: 'input_5',
          },
          {
            type: 'image',
            image_id: 'compositor_icon',
          },
          {
            type: 'shader',
            shader_id: 'red_border',
            shader_param: {
              type: 'list',
              value: [
                {
                  type: 'u32',
                  value: 0,
                },
                {
                  type: 'u32',
                  value: 128,
                },
                {
                  type: 'u32',
                  value: 255,
                },
                {
                  type: 'u32',
                  value: 255,
                },
              ],
            },
            children: [
              {
                type: 'image',
                image_id: 'landscape',
              },
            ],
            resolution: {
              width: 1920,
              height: 1080,
            },
          },
        ],
      },
    ],
  };
}
