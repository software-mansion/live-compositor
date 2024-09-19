import { initializeCompositor } from './compositor';
import { app } from './routes';

async function run() {
  await initializeCompositor();

  app.listen(3000);
}

void run();
