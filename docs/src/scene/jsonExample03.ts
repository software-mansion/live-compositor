import { Api } from 'live-compositor';

export function example03(): Api.Component {
  return {
    type: 'view',
    background_color_rgba: '#ffc89ad9',
    children: [
      {
        type: 'rescaler',
        child: {
          type: 'shader',
          shader_id: 'remove_greenscreen',
          children: [
            {
              type: 'image',
              image_id: 'greenscreen',
            },
          ],
          resolution: {
            width: 2160,
            height: 2880,
          },
        },
      },
      {
        type: 'rescaler',
        top: 20,
        left: 20,
        width: 350,
        height: 200,
        child: {
          type: 'image',
          image_id: 'compositor_icon',
        },
      },
    ],
  };
}
