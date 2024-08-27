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
        name="bunny"
        description={
          <div>
            Big Buck Bunny screenshot, <code>[16:9] 1280x720</code> resolution
          </div>
        }
      />
      <ImagePreview
        name="landscape"
        description={
          <div>
            Landscape photo, <code>[4:3] 2560x1920</code> resolution
          </div>
        }
      />
      <ImagePreview
        name="person"
        description={
          <div>
            Person photo, can be used ex. as a videocall substitute, <code>[3:2] 4096x2731</code>{' '}
            resolution
          </div>
        }
      />
      <ImagePreview
        name="test_pattern"
        description={
          <div>
            FFmpeg-generated test source, <code>[16:9] 7680x4320</code> resolution
          </div>
        }
      />
    </div>
  );
}

interface ImagePreviewProps {
  name: string;
  description: JSX.Element;
}

function ImagePreview({ name, description }: ImagePreviewProps) {
  const tooltipJson = { type: 'image', input_id: name };

  return (
    <div className={styles.imagePreview}>
      <div className={styles.imagePreviewLabelContainer}>
        <code id={`${name}_tooltip`}>{name}</code>
      </div>
      <div className={styles.imagePreviewDescriptionContainer}>{description}</div>
      <div className={styles.imagePreviewImgContainer}>
        <img src={getImagePath(name)} alt={'alt'} className={styles.imagePreviewImg} />
      </div>
      <Tooltip
        anchorSelect={`#${name}_tooltip`}
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

function getImagePath(inputName: string): string {
  return `/img/images/${inputName}.webp`;
}
