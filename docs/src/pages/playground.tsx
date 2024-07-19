import clsx from 'clsx';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';

import styles from './index.module.css';

function Homepage() {
  return (
    <div className={clsx('container', styles.firstSection)}>
      <div className="row">
        <div>
          <textarea name="post" rows={40} cols={100} />
        </div>
        <div className={styles.heroVideo}>
          <div
            style={{
              borderRadius: '15px',
              overflow: 'hidden',
              boxShadow: '0px 0px 20px 1px #4332a6',
            }}>
            <video
              loop
              autoPlay
              muted
              src="/video/showcase.mp4"
              poster="/img/showcase_poster.jpg"
              style={{ width: '100%', display: 'block' }}
            />
          </div>
        </div>
      </div>
    </div>
  );
}

export default function Home(): JSX.Element {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout
      title={siteConfig.title}
      description="Tool for real-time video processing / transforming / composing">
      <Homepage />
    </Layout>
  );
}
