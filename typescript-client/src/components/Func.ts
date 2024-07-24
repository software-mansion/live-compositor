import * as Api from '../api';
import { Component, FunctionComponent } from '../component';
import { removeHookState, setupHookState } from '../hookState';
import { RenderContext } from '../context';

class Func<Props> extends Component<Props> {
  props: Props;
  fn: FunctionComponent<Props>;

  constructor(fn: FunctionComponent<Props>, props: Props) {
    super();
    this.props = props;
    this.fn = fn;
  }

  scene(ctx: RenderContext): Api.Component {
    setupHookState(ctx);
    const component = this.fn(this.props);
    removeHookState();
    return component.scene(ctx);
  }

  update(props: Props): void {
    this.props = props;
  }
}

export default Func;
