import toast from 'react-hot-toast';
import { Tooltip } from 'react-tooltip';
import styles from './PlaygroundSettingsShaders.module.css';

export default function PlaygroundSettingsShaders() {
  return (
    <div className={styles.container}>
      <div className={styles.headerContainer}>
        <div className={styles.headerInputLabel}>Shader ID</div>
        <div className={styles.headerDescriptionLabel}>Description</div>
      </div>
      <ShaderInfo
        shader_id="remove_greenscreen"
        description="Shader removing a green background. It supports exactly one child component."
        tooltipJson={`{
  "type": "shader",
  "shader_id": "remove_greenscreen",
  "children": [
    { "type": "image", "image_id": "greenscreen" }
  ],
  "resolution": { "width": 2160, "height": 2880 }
}`}
      />
      <ShaderInfo
        shader_id="red_border"
        description="Shader that adds a red border around the child component. It supports exactly one child component and takes color as a param (4-element list of u32 RGBA values from 0 to 255)."
        tooltipJson={`{
  "type": "shader",
  "shader_id": "red_border",
  "shader_param": {
    "type": "list",
    "value": [{ "type": "u32", "value": 0 }, { "type": "u32", "value": 128 }, { "type": "u32", "value": 255 }, { "type": "u32", "value": 255 }]
  },
  "children": [
    { "type": "image", "image_id": "landscape" }
  ],
  "resolution": { "width": 1920, "height": 1080 }
}`}
      />
      <ShaderInfo
        shader_id="rounded_corners"
        description="Shader that implements rounded corners. It supports exactly one child component and takes radius as a param (f32 value)."
        tooltipJson={`{
  "type": "shader",
  "shader_id": "rounded_corners",
  "shader_param": { "type": "f32", "value": 64 },
  "children": [
    { "type": "image", "image_id": "person" }
  ],
  "resolution": { "width": 3000, "height": 2000 }
}`}
      />
    </div>
  );
}

interface ShaderInfoProps {
  shader_id: string;
  description: string;
  tooltipJson: string;
}

function ShaderInfo({ shader_id, description, tooltipJson }: ShaderInfoProps) {
  return (
    <div className={styles.shaderInfo}>
      <div className={styles.shaderInfoLabelContainer}>
        <code id={`${shader_id}_tooltip`}>{shader_id}</code>
      </div>
      <div className={styles.shaderInfoDescriptionContainer}>{description}</div>
      <Tooltip
        anchorSelect={`#${shader_id}_tooltip`}
        className={styles.tooltip}
        clickable={true}
        delayShow={128}
        positionStrategy="fixed">
        <div style={{ maxWidth: '88vw' }}>
          {`Add `}
          <code
            className={styles.tooltipCode}
            onClick={async () => {
              await navigator.clipboard.writeText(tooltipJson);
              toast.success('Copied to clipboard!');
            }}>
            {tooltipJson}
          </code>
          {` to use this shader.`}
        </div>
      </Tooltip>
    </div>
  );
}
