interface PlaygroundPreviewProps {
  imageUrl?: string;
  errorMessage?: string;
}

function PlaygroundPreview({ imageUrl, errorMessage }: PlaygroundPreviewProps) {
  if (errorMessage) {
    return <div>{errorMessage}</div>;
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
  } else {
    return null;
  }
}

export default PlaygroundPreview;
