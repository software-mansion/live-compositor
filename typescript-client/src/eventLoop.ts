import LiveCompositor from './compositor';
import { isRegisterInterval, isRegisterPts, RegisterEvent, RenderContext } from './context';
import { Context } from './types';

export class LiveCompositorState {
  private startTimestampMs: number | null = null;
  private withIntervalsMs: number[] = [];
  private withPtsMs: number[] = [];

  // TODO: figure out node and browser types
  private currentSetTimeout: any | null = null;

  constructor() {}

  public start() {
    this.startTimestampMs = Date.now();
  }

  public initRenderContext(ctx: Context): {
    renderContext: RenderContext;
    contextDone: () => void;
  } {
    const withIntervalsMs: number[] = [];
    const withPtsMs: number[] = [];

    const renderContext = {
      publicContext: ctx,
      registerCallback: (event: RegisterEvent) => {
        if (isRegisterPts(event)) {
          withPtsMs.push(event.pts);
        } else if (isRegisterInterval(event)) {
          withIntervalsMs.push(event.intervalMs);
        } else {
          console.error(`Unknown event ${event}`);
        }
      },
    };

    return {
      renderContext: renderContext,
      contextDone: () => {
        this.withPtsMs = withPtsMs;
        this.withIntervalsMs = withIntervalsMs;
      },
    };
  }

  public maybeScheduleNextUpdate(compositor: LiveCompositor<any>) {
    const nextUpdateTimeout = this.nextUpdateTimeout();
    if (nextUpdateTimeout === Infinity) {
      return;
    }
    if (this.currentSetTimeout != null) {
      clearTimeout(this.currentSetTimeout);
    }
    this.currentSetTimeout = setTimeout(() => {
      compositor.render();
    }, nextUpdateTimeout);
  }

  private nextUpdateTimeout(): number {
    const now = Date.now();

    const withIntervalsTimeouts = this.withIntervalsMs.map(intervalMs => {
      const intervalsSinceStart = (now - this.startTimestampMs!) / intervalMs;
      const passedIntervalFraction = intervalsSinceStart - Math.floor(intervalsSinceStart);
      return (1 - passedIntervalFraction) * intervalMs;
    });

    const withPtsTimeouts = this.withPtsMs.map(pts => {
      const currentPts = now - this.startTimestampMs!;
      return currentPts < pts ? pts - currentPts : Infinity;
    });

    return Math.min(...withIntervalsTimeouts, ...withPtsTimeouts);
  }
}
