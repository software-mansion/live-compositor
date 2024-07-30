interface PlaygroundPreviewProps {
  responseData: {
    imageUrl: string;
    errorMessage: string;
  };
}

function PlaygroundPreview({ responseData }: PlaygroundPreviewProps): JSX.Element {
  if (responseData.errorMessage) {
    return <div>{responseData.errorMessage}</div>;
  }
  if (responseData.imageUrl) {
    return (
      <img
        src={responseData.imageUrl}
        style={{
          objectFit: 'contain',
          height: '100%',
          width: '100%',
        }}
      />
    );
  }
  return null;
}

export default PlaygroundPreview;
