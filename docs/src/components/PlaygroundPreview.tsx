import { RotatingLines } from 'react-loader-spinner';
interface PlaygroundPreviewProps {
  imageUrl?: string;
  errorMessage?: string;
  loading?: boolean;
}

function PlaygroundPreview({ imageUrl, errorMessage, loading }: PlaygroundPreviewProps) {
  if (errorMessage) {
    return <div style={{ alignContent: 'center', margin: '20px' }}>{errorMessage}</div>;
  } else if (imageUrl) {
    return (
      <img
        src={imageUrl}
        style={{
          objectFit: 'contain',
          height: '100%',
          width: '100%',
        }}
      />
    );
  } else if (loading) {
    return (
      <div style={{ alignContent: 'center' }}>
        <RotatingLines
          visible={true}
          width="96"
          strokeColor="grey"
          strokeWidth="5"
          animationDuration="0.5"
          ariaLabel="rotating-lines-loading"
        />
      </div>
    );
  } else {
    return null;
  }
}

export default PlaygroundPreview;
