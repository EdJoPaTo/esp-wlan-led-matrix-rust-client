use tokio::io::{AsyncReadExt, AsyncWriteExt, BufStream};
use tokio::net::{TcpStream, ToSocketAddrs};

use crate::Command;

pub struct Client {
    stream: BufStream<TcpStream>,
    width: u8,
    height: u8,
}

impl Client {
    /// Connect to the server
    ///
    /// # Errors
    /// Errors when the connection could not be established.
    pub async fn connect<A>(addr: A) -> std::io::Result<Self>
    where
        A: ToSocketAddrs + Send,
    {
        let stream = TcpStream::connect(addr).await?;
        let mut stream = BufStream::new(stream);

        let mut buf = [0; 2];
        stream.read_exact(&mut buf).await?;
        let [width, height] = buf;

        Ok(Self {
            stream,
            width,
            height,
        })
    }

    #[must_use]
    pub const fn width(&self) -> u8 {
        self.width
    }

    #[must_use]
    pub const fn height(&self) -> u8 {
        self.height
    }

    #[must_use]
    pub const fn total_pixels(&self) -> u16 {
        (self.width as u16) * (self.height as u16)
    }

    /// Flushes the internal buffer and sends everything to the server
    ///
    /// # Errors
    /// Errors when the command could not be sent
    pub async fn flush(&mut self) -> std::io::Result<()> {
        self.stream.flush().await
    }

    /// Set one pixel of the matrix to the given color.
    /// Do not forget to also run [flush] afterwards.
    ///
    /// # Errors
    /// Errors when the data could not be written to the send buffer
    ///
    /// [flush]: Self::flush
    pub async fn pixel(
        &mut self,
        x: u8,
        y: u8,
        red: u8,
        green: u8,
        blue: u8,
    ) -> std::io::Result<()> {
        self.stream
            .write_all(&[Command::Pixel as u8, x, y, red, green, blue])
            .await
    }

    /// Fill the whole matrix with one color.
    /// Do not forget to also run [flush] afterwards.
    ///
    /// # Errors
    /// Errors when the command could not be sent
    ///
    /// [flush]: Self::flush
    pub async fn fill(&mut self, red: u8, green: u8, blue: u8) -> std::io::Result<()> {
        self.stream
            .write_all(&[Command::Fill as u8, red, green, blue])
            .await
    }

    #[allow(clippy::too_many_arguments)]
    /// Fill the given rectangular area with one color.
    /// Do not forget to also run [flush] afterwards.
    ///
    /// # Errors
    /// Errors when the command could not be sent
    ///
    /// [flush]: Self::flush
    pub async fn rectangle(
        &mut self,
        x: u8,
        y: u8,
        width: u8,
        height: u8,
        red: u8,
        green: u8,
        blue: u8,
    ) -> std::io::Result<()> {
        self.stream
            .write_all(&[
                Command::Rectangle as u8,
                x,
                y,
                width,
                height,
                red,
                green,
                blue,
            ])
            .await
    }

    /// Send an area full of colors.
    ///
    /// The area begins in the top left at x/y and moves first on the x axis, then on the y axis.
    /// The colors are given in R G B order.
    ///
    /// Do not forget to also run [flush] afterwards.
    ///
    /// # Errors
    /// Errors when the command could not be sent
    ///
    /// [flush]: Self::flush
    pub async fn contiguous(
        &mut self,
        x: u8,
        y: u8,
        width: u8,
        height: u8,
        colors: &[u8],
    ) -> std::io::Result<()> {
        let too_wide = x.checked_add(width).map_or(true, |w| w > self.width);
        let too_high = y.checked_add(height).map_or(true, |h| h > self.height);
        if too_wide || too_high {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "area too big for display",
            ));
        }

        let expected_length = (width as usize) * (height as usize) * 3;
        if expected_length != colors.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "colors is wrong length",
            ));
        }

        self.stream
            .write_all(&[Command::Contiguous as u8, x, y, width, height])
            .await?;
        self.stream.write_all(colors).await
    }
}
