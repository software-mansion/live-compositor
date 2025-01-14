import React, { useCallback, useEffect, useState } from 'react';
import { LiveCompositor } from '@live-compositor/web-wasm';

type CanvasProps = React.DetailedHTMLProps<
  React.CanvasHTMLAttributes<HTMLCanvasElement>,
  HTMLCanvasElement
>;

type CompositorCanvasProps = {
  onCanvasCreate?: (compositor: LiveCompositor) => Promise<void>;
  onCanvasStarted?: (compositor: LiveCompositor) => Promise<void>;
  children: React.ReactElement;
} & CanvasProps;

export default function CompositorCanvas(props: CompositorCanvasProps) {
  const { onCanvasCreate, onCanvasStarted, children, ...canvasProps } = props;

  const [compositor, setCompositor] = useState<LiveCompositor | undefined>(undefined);
  const canvasRef = useCallback(
    async (canvas: HTMLCanvasElement | null) => {
      if (!canvas) {
        return;
      }
      const compositor = new LiveCompositor({});

      await compositor.init();

      if (onCanvasCreate) {
        await onCanvasCreate(compositor);
      }

      await compositor.registerOutput('output', children, {
        type: 'canvas',
        canvas: canvas,
        resolution: {
          width: Number(canvasProps.width ?? canvas.width),
          height: Number(canvasProps.height ?? canvas.height),
        },
      });

      await compositor.start();
      setCompositor(compositor);

      if (onCanvasStarted) {
        await onCanvasStarted(compositor);
      }
    },
    [onCanvasCreate, onCanvasStarted, canvasProps.width, canvasProps.height, children]
  );

  useEffect(() => {
    return () => {
      if (compositor) {
        void compositor.terminate();
      }
    };
  }, [compositor]);

  return <canvas ref={canvasRef} {...canvasProps}></canvas>;
}
