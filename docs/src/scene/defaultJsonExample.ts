import { Api } from 'live-compositor';

export function defaultJsonExample(): Api.Component {
  return {
    type: 'view',
    background_color: '#4d4d4dff',
    children: [
      {
        type: 'view',
        children: [
          {
            type: 'rescaler',
            top: 0,
            left: 0,
            mode: 'fill',
            child: {
              type: 'image',
              image_id: 'landscape',
            },
          },
          {
            type: 'rescaler',
            child: {
              type: 'shader',
              shader_id: 'remove_greenscreen',
              children: [{ type: 'image', image_id: 'greenscreen' }],
              resolution: { width: 2160, height: 2880 },
            },
          },
          {
            type: 'rescaler',
            top: 30,
            left: 30,
            width: 360,
            height: 270,
            vertical_align: 'top',
            child: {
              type: 'shader',
              shader_id: 'rounded_corners',
              shader_param: { type: 'f32', value: 30 },
              children: [{ type: 'input_stream', input_id: 'input_1' }],
              resolution: { width: 360, height: 203 },
            },
          },
        ],
      },
      {
        type: 'tiles',
        width: 600,
        padding: 20,
        children: [
          { type: 'input_stream', input_id: 'input_2' },
          { type: 'input_stream', input_id: 'input_3' },
          { type: 'input_stream', input_id: 'input_4' },
          { type: 'input_stream', input_id: 'input_5' },
          { type: 'input_stream', input_id: 'input_6' },
          { type: 'image', image_id: 'greenscreen' },
          { type: 'image', image_id: 'test_pattern' },
        ],
      },
    ],
  };
}
