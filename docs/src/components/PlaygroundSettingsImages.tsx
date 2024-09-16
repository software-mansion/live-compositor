import toast from 'react-hot-toast';
import { Tooltip } from 'react-tooltip';
import styles from './PlaygroundSettingsImages.module.css';

export default function PlaygroundSettingsImages() {
  return (
    <div className={styles.container}>
      <div className={styles.imagePreviewIntro}>
        List of available images. You can use them in the scene by adding{' '}
        <code>{'{ "type": "image"; "image_id": "[image_id]" }'}</code> in&nbsp;the&nbsp;JSON
        definition.
      </div>
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
            Screenshot from Big Buck Bunny video, <code>[16:9] 1280x720</code> resolution.
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
            Photo of a person speaking to a camera, <code>[3:2] 3000x2000</code> resolution.
          </div>
        }
      />
      <ImagePreview
        image_id="greenscreen"
        filename="greenscreen.webp"
        description={
          <div>
            Photo of a person with a green background, <code>2160x2880</code> resolution. Example
            shader <code>remove_greenscreen</code> can be used to remove the background from the
            image.
          </div>
        }
      />
      <ImagePreview
        image_id="test_pattern"
        filename="test_pattern.webp"
        description={
          <div>
            Example test pattern, <code>[16:9] 1920x1080</code> resolution.
          </div>
        }
      />
      <ImagePreview
        image_id="compositor_logo"
        filename="compositor_logo.svg"
        description={
          <div>
            SVG of the LiveCompositor logo with text, <code>572x140</code> resolution.
          </div>
        }
      />
      <ImagePreview
        image_id="compositor_icon"
        filename="compositor_icon.webp"
        description={
          <div>
            PNG of the LiveCompositor logo with an alpha channel, <code>200x140</code> resolution.
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
  const json = JSON.stringify({ type: 'image', image_id: image_id }, null, 2);

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
            onClick={async () => {
              await navigator.clipboard.writeText(json);
              toast.success('Copied to clipboard!');
            }}>
            {json}
          </code>
          {` to use this image.`}
        </div>
      </Tooltip>
    </div>
  );
}

function getImagePath(filename: string): string {
  return `/img/playground_images/${filename}`;
}
