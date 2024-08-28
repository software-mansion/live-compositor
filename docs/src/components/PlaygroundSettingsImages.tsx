import toast from 'react-hot-toast';
import { Tooltip } from 'react-tooltip';
import styles from './PlaygroundSettingsImages.module.css';

export default function PlaygroundSettingsImages() {
  return (
    <div className={styles.container}>
      <div className={styles.headerContainer}>
        <div className={styles.headerInputLabel}>Image ID</div>
        <div className={styles.headerDescriptionLabel}>Description</div>
        <div className={styles.headerPreviewLabel}>Preview</div>
      </div>
      <ImagePreview
        image_id="bunny"
        filename="bunny.webp"
        description={
          <div>
            Big Buck Bunny screenshot, <code>[16:9] 1280x720</code> resolution.
          </div>
        }
      />
      <ImagePreview
        image_id="landscape"
        filename="landscape.webp"
        description={
          <div>
            Landscape photo, <code>[16:9] 2560x1440</code> resolution.
          </div>
        }
      />
      <ImagePreview
        image_id="person"
        filename="person.webp"
        description={
          <div>
            Person photo, can be used e.g. as a videocall substitute, <code>[3:2] 4096x2731</code>{' '}
            resolution.
          </div>
        }
      />
      <ImagePreview
        image_id="greenscreen_guy"
        filename="greenscreen_guy.webp"
        description={
          <div>
            Photo of guy on greenscreen, can be used to demo <code>remove_greenscreen</code> shader,{' '}
            <code>2160x2880</code> resolution.
          </div>
        }
      />
      <ImagePreview
        image_id="test_pattern"
        filename="test_pattern.webp"
        description={
          <div>
            FFmpeg-generated test source, <code>[16:9] 7680x4320</code> resolution.
          </div>
        }
      />
      <ImagePreview
        image_id="compositor"
        filename="compositor.svg"
        description={
          <div>
            Svg compositor logo with alpha channel, <code>572x140</code> resolution.
          </div>
        }
      />
      <ImagePreview
        image_id="compositor_small"
        filename="compositor_small.webp"
        description={
          <div>
            Png small compositor logo alpha channel, <code>200x140</code> resolution.
          </div>
        }
      />
    </div>
  );
}

interface ImagePreviewProps {
  image_id: string;
  filename: string;
  description: JSX.Element;
}

function ImagePreview({ image_id, description, filename }: ImagePreviewProps) {
  const tooltipJson = { type: 'image', image_id: image_id };

  return (
    <div className={styles.imagePreview}>
      <div className={styles.imagePreviewLabelContainer}>
        <code id={`${image_id}_tooltip`}>{image_id}</code>
      </div>
      <div className={styles.imagePreviewDescriptionContainer}>{description}</div>
      <div className={styles.imagePreviewImgContainer}>
        <img src={getImagePath(filename)} alt={'alt'} className={styles.imagePreviewImg} />
      </div>
      <Tooltip
        anchorSelect={`#${image_id}_tooltip`}
        className={styles.tooltip}
        clickable={true}
        delayShow={128}>
        <div style={{ maxWidth: '88vw' }}>
          {`Add `}
          <code
            className={styles.tooltipCode}
            onClick={() => {
              navigator.clipboard.writeText(JSON.stringify(tooltipJson));
              toast.success('Copied to clipboard!');
            }}>
            {JSON.stringify(tooltipJson)}
          </code>
          {` to use this image.`}
        </div>
      </Tooltip>
    </div>
  );
}

function getImagePath(filename: string): string {
  return `/img/images/${filename}`;
}
