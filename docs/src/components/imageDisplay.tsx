const ImageDisplay = ({ imageUrl }) => (
  <div>
    {imageUrl ? (
      <img
        src={imageUrl}
        style={{
          objectFit: 'contain',
          height: '100%',
          width: '100%',
        }}
      />
    ) : (
      <div></div>
    )}
  </div>
);

export default ImageDisplay;
