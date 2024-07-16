export const ScreenViewer = (props: { text: string | null }) => {
  return (
    <div className="grid border-2 border-yellow-500 border-dotted h-full w-full">
      <div className="grid border-2 border-red-500 place-items-center bg-black">
        <div className="text-center text-4xl text-white outline-black outline-4">
          {String(props.text || "")}
        </div>
      </div>
    </div>
  );
};
